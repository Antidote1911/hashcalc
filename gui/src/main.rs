use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc};

use eframe::egui::{self, Color32, RichText, Stroke, StrokeKind, Vec2};
use hashcalc_core::Algorithm;
use strum::IntoEnumIterator;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([680.0, 400.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "HashCalc",
        options,
        Box::new(|_cc| Ok(Box::new(HashCalcApp::default()))),
    )
}

struct HashCalcApp {
    file_path: Option<PathBuf>,
    algorithm: Algorithm,
    hash_result: Option<String>,
    error: Option<String>,
    /// Channel receiver for the background thread result.
    computing: Option<mpsc::Receiver<Result<String, String>>>,
    /// Bytes processed by the current computation (new Arc each run).
    progress: Arc<AtomicU64>,
    /// Total file size in bytes (0 = unknown).
    file_size: u64,
    dark_mode: bool,
}

impl Default for HashCalcApp {
    fn default() -> Self {
        Self {
            file_path: None,
            algorithm: Algorithm::default(),
            hash_result: None,
            error: None,
            computing: None,
            progress: Arc::new(AtomicU64::new(0)),
            file_size: 0,
            dark_mode: true,
        }
    }
}

impl HashCalcApp {
    fn set_file(&mut self, path: PathBuf) {
        self.file_path = Some(path);
        self.hash_result = None;
        self.error = None;
        self.computing = None;
    }

    fn start_hash(&mut self, ctx: &egui::Context) {
        let Some(path) = self.file_path.clone() else {
            return;
        };

        // Each run gets its own atomic so stale threads can't corrupt the new bar.
        let progress = Arc::new(AtomicU64::new(0));
        self.progress = Arc::clone(&progress);
        self.file_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        self.hash_result = None;
        self.error = None;

        let (tx, rx) = mpsc::channel();
        let hasher = self.algorithm.into_hasher_with_progress();
        let ctx = ctx.clone();

        std::thread::spawn(move || {
            let result = hasher(&path, &progress).map_err(|e| e.to_string());
            let _ = tx.send(result);
            ctx.request_repaint();
        });

        self.computing = Some(rx);
    }

    fn progress_fraction(&self) -> f32 {
        if self.file_size == 0 {
            return 0.0;
        }
        (self.progress.load(Ordering::Relaxed) as f32 / self.file_size as f32).min(1.0)
    }
}

impl eframe::App for HashCalcApp {
    // Called before ui() every frame — set visuals here so the CentralPanel
    // background is drawn with the correct theme on the same frame.
    #[allow(deprecated)]
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(if self.dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        });
    }

    // Override the GPU clear color so the area behind egui panels is never
    // left as the default near-black when switching to light mode.
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        if self.dark_mode {
            egui::Visuals::dark().panel_fill.to_normalized_gamma_f32()
        } else {
            egui::Visuals::light().panel_fill.to_normalized_gamma_f32()
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // --- Poll background computation ---
        if let Some(rx) = &self.computing {
            match rx.try_recv() {
                Ok(Ok(hash)) => {
                    self.hash_result = Some(hash);
                    self.computing = None;
                }
                Ok(Err(e)) => {
                    self.error = Some(e);
                    self.computing = None;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint();
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.computing = None;
                }
            }
        }

        // --- Handle drag-and-drop ---
        ui.input(|i| {
            for file in &i.raw.dropped_files {
                if let Some(path) = file.path.clone() {
                    if path.is_file() {
                        self.set_file(path);
                    }
                }
            }
        });

        // Trigger computation when a file is dropped.
        // (set_file clears computing, so this check is safe.)
        if self.file_path.is_some() && self.computing.is_none() && self.hash_result.is_none() && self.error.is_none() {
            self.start_hash(&ctx);
        }

        let is_dragging = ui.input(|i| !i.raw.hovered_files.is_empty());
        let is_computing = self.computing.is_some();

        // ── Header ──────────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.heading("HashCalc");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let icon = if self.dark_mode { "☀" } else { "🌙" };
                if ui.button(icon).on_hover_text(if self.dark_mode {
                    "Passer en mode clair"
                } else {
                    "Passer en mode sombre"
                }).clicked() {
                    self.dark_mode = !self.dark_mode;
                }
            });
        });
        ui.separator();
        ui.add_space(10.0);

        // ── Algorithm selector ───────────────────────────────────────────────
        let prev_algorithm = self.algorithm;
        ui.horizontal(|ui| {
            ui.label("Algorithme :");
            egui::ComboBox::from_id_salt("algo_selector")
                .selected_text(self.algorithm.to_string())
                .width(200.0)
                .show_ui(ui, |ui| {
                    for algo in Algorithm::iter() {
                        ui.selectable_value(&mut self.algorithm, algo, algo.to_string());
                    }
                });
        });

        // Auto-trigger when the algorithm changes and a file is already loaded.
        if self.algorithm != prev_algorithm {
            if self.file_path.is_some() {
                self.start_hash(&ctx);
            }
        }

        ui.add_space(14.0);

        // ── Drop zone ───────────────────────────────────────────────────────
        let available_width = ui.available_width();
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(available_width, 90.0), egui::Sense::click());

        if response.clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                self.set_file(path);
                self.start_hash(&ctx);
            }
        }

        if ui.is_rect_visible(rect) {
            let stroke_color = if is_dragging {
                Color32::from_rgb(80, 200, 80)
            } else if response.hovered() {
                ui.visuals().widgets.hovered.bg_stroke.color
            } else {
                ui.visuals().widgets.noninteractive.bg_stroke.color
            };

            let fill = if is_dragging {
                Color32::from_rgba_premultiplied(80, 200, 80, 18)
            } else if response.hovered() {
                ui.visuals().widgets.hovered.weak_bg_fill
            } else {
                Color32::TRANSPARENT
            };

            let painter = ui.painter();
            painter.rect_filled(rect, egui::CornerRadius::same(8), fill);
            painter.rect_stroke(
                rect,
                egui::CornerRadius::same(8),
                Stroke::new(if is_dragging { 2.0 } else { 1.5 }, stroke_color),
                StrokeKind::Middle,
            );

            let label = if is_dragging {
                "📂  Déposer le fichier…".to_string()
            } else if let Some(p) = &self.file_path {
                format!(
                    "📄  {}",
                    p.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("fichier sélectionné")
                )
            } else {
                "📂  Déposer un fichier ici ou cliquer pour parcourir".to_string()
            };

            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(15.0),
                ui.visuals().text_color(),
            );
        }

        ui.add_space(6.0);

        // Full path (small, muted)
        if let Some(path) = &self.file_path {
            ui.label(
                RichText::new(path.display().to_string())
                    .small()
                    .color(ui.visuals().weak_text_color()),
            );
        }

        ui.add_space(10.0);

        // ── Progress bar ────────────────────────────────────────────────────
        if is_computing {
            let fraction = self.progress_fraction();
            ui.add(
                egui::ProgressBar::new(fraction)
                    .desired_width(available_width)
                    .animate(fraction == 0.0),
            );
            ui.add_space(6.0);
        } else {
            // Reserve the same vertical space to avoid layout jumps.
            ui.add_space(20.0);
            ui.add_space(6.0);
        }

        // ── Hash result ─────────────────────────────────────────────────────
        if let Some(hash) = self.hash_result.clone() {
            ui.label(RichText::new("Résultat :").strong());
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut hash.as_str())
                        .desired_width(ui.available_width() - 90.0)
                        .font(egui::TextStyle::Monospace),
                );
                if ui.button("📋 Copier").clicked() {
                    ctx.copy_text(hash.clone());
                }
            });
        }

        // ── Error ────────────────────────────────────────────────────────────
        if let Some(err) = &self.error {
            ui.colored_label(Color32::RED, format!("Erreur : {err}"));
        }

        // ── Full-window drag overlay ─────────────────────────────────────────
        if is_dragging {
            if let Some(viewport) =
                ctx.input(|i| i.viewport().inner_rect.or(i.viewport().outer_rect))
            {
                ui.painter().rect_stroke(
                    viewport.shrink(4.0),
                    egui::CornerRadius::same(4),
                    Stroke::new(3.0, Color32::from_rgb(80, 200, 80)),
                    StrokeKind::Middle,
                );
            }
        }
    }
}
