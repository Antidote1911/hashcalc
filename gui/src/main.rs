use std::{
    fs::File,
    io::{self, Result as IoResult, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    sync::{mpsc, Arc},
};

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use eframe::egui::{self, Color32, RichText, Stroke, StrokeKind, Vec2};
use hashcalc_core::{hashfile::HashFile, Algorithm, Mode};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use strum::IntoEnumIterator;

// ── CLI ──────────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(about, author, version, arg_required_else_help(true))]
struct Opt {
    /// Text or binary mode.
    #[arg(short, long, value_enum, default_value_t)]
    mode: Mode,
    /// Hashing algorithm.
    #[arg(short, long, value_enum, default_value_t)]
    algorithm: Algorithm,
    /// Prepend each output line with the used hash algorithm.
    #[arg(short, long)]
    prefix: bool,
    /// Files to hash. Directories are silently ignored.
    files: Vec<PathBuf>,
    #[command(subcommand)]
    cmd: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Check hashes against their actual files.
    Check {
        /// File containing the list of hashes.
        file: PathBuf,
    },
    /// Generate shell completions, writing them to stdout.
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Generate man pages into the given directory.
    Manpages {
        dir: PathBuf,
    },
}

const IS_A_DIRECTORY: i32 = 21;

fn main() {
    if std::env::args_os().len() <= 1 {
        if let Err(e) = run_gui() {
            eprintln!("{e}");
            std::process::exit(1);
        }
    } else if let Err(e) = run_cli() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run_cli() -> Result<()> {
    let opt = Opt::parse();

    if let Some(cmd) = opt.cmd {
        match cmd {
            Command::Check { file } => verify_files(file),
            Command::Completions { shell } => {
                print_completions(shell);
                Ok(())
            }
            Command::Manpages { dir } => print_manpages(&dir),
        }
    } else {
        hash_files(opt)
    }
}

fn verify_files(file: PathBuf) -> Result<()> {
    let hash_file = HashFile::parse(file)?;

    let matches = hash_file
        .entries
        .into_par_iter()
        .map(|entry| {
            let hasher = entry
                .algorithm
                .or(hash_file.algorithm)
                .unwrap_or(Algorithm::Blake3)
                .into_hasher();

            let hash = hasher(Path::new(&entry.file))?;

            Ok((entry.file, hash == entry.hash))
        })
        .collect::<Result<Vec<_>>>()?;

    let width = matches
        .iter()
        .map(|(file, _)| file.len())
        .max()
        .unwrap_or_default();

    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    for (file, valid) in matches {
        writeln!(
            stdout,
            "{:width$} {}",
            file,
            if valid { "OK" } else { "ERR" },
            width = width
        )?;
    }

    stdout.flush()?;
    Ok(())
}

fn print_completions(shell: Shell) {
    clap_complete::generate(
        shell,
        &mut Opt::command(),
        env!("CARGO_PKG_NAME"),
        &mut io::stdout().lock(),
    );
}

fn print_manpages(dir: &Path) -> Result<()> {
    fn print(dir: &Path, app: &clap::Command) -> Result<()> {
        let name = app.get_display_name().unwrap_or_else(|| app.get_name());
        let mut out = File::create(dir.join(format!("{name}.1")))?;
        clap_mangen::Man::new(app.clone()).render(&mut out)?;
        out.flush()?;
        for sub in app.get_subcommands() {
            print(dir, sub)?;
        }
        Ok(())
    }

    let mut app = Opt::command();
    app.build();
    print(dir, &app)
}

fn hash_files(opt: Opt) -> Result<()> {
    let hasher = opt.algorithm.into_hasher();

    let mut hashes = opt
        .files
        .into_par_iter()
        .filter_map(|file| match hasher(&file) {
            Ok(hash) => Some(Ok((hash, file))),
            Err(e) if e.raw_os_error() == Some(IS_A_DIRECTORY) => None,
            Err(e) => Some(Err(e)),
        })
        .collect::<IoResult<Vec<_>>>()?;

    hashes.sort_unstable_by(|(_, a), (_, b)| a.cmp(b));

    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    for (hash, path) in hashes {
        if opt.prefix {
            write!(stdout, "{} ", opt.algorithm)?;
        }
        writeln!(stdout, "{} {}{}", hash, opt.mode, path.display())?;
    }

    stdout.flush()?;
    Ok(())
}

// ── GUI ──────────────────────────────────────────────────────────────────────

fn run_gui() -> eframe::Result {
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
    computing: Option<mpsc::Receiver<Result<String, String>>>,
    progress: Arc<AtomicU64>,
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
    #[allow(deprecated)]
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(if self.dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        });
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        if self.dark_mode {
            egui::Visuals::dark().panel_fill.to_normalized_gamma_f32()
        } else {
            egui::Visuals::light().panel_fill.to_normalized_gamma_f32()
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

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

        ui.input(|i| {
            for file in &i.raw.dropped_files {
                if let Some(path) = file.path.clone() {
                    if path.is_file() {
                        self.set_file(path);
                    }
                }
            }
        });

        if self.file_path.is_some() && self.computing.is_none() && self.hash_result.is_none() && self.error.is_none() {
            self.start_hash(&ctx);
        }

        let is_dragging = ui.input(|i| !i.raw.hovered_files.is_empty());
        let is_computing = self.computing.is_some();

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

        if self.algorithm != prev_algorithm {
            if self.file_path.is_some() {
                self.start_hash(&ctx);
            }
        }

        ui.add_space(14.0);

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

        if let Some(path) = &self.file_path {
            ui.label(
                RichText::new(path.display().to_string())
                    .small()
                    .color(ui.visuals().weak_text_color()),
            );
        }

        ui.add_space(10.0);

        if is_computing {
            let fraction = self.progress_fraction();
            ui.add(
                egui::ProgressBar::new(fraction)
                    .desired_width(available_width)
                    .animate(fraction == 0.0),
            );
            ui.add_space(6.0);
        } else {
            ui.add_space(20.0);
            ui.add_space(6.0);
        }

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

        if let Some(err) = &self.error {
            ui.colored_label(Color32::RED, format!("Erreur : {err}"));
        }

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
