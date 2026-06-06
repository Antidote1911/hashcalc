#include "MainWindow.h"

#include <QApplication>
#include <QClipboard>
#include <QFileDialog>
#include <QFileInfo>
#include <QFont>
#include <QFontDatabase>
#include <QFrame>
#include <QHBoxLayout>
#include <QLabel>
#include <QMetaObject>
#include <QPalette>
#include <QVBoxLayout>
#include <QWidget>

#include <hashcalc_ffi.h>

#include <thread>

MainWindow::MainWindow(QWidget* parent) : QMainWindow(parent) {
    setWindowTitle(QStringLiteral("HashCalc"));
    resize(700, 430);

    auto* central    = new QWidget;
    auto* mainLayout = new QVBoxLayout(central);
    mainLayout->setContentsMargins(16, 16, 16, 16);
    mainLayout->setSpacing(10);
    setCentralWidget(central);

    // ── Title bar ─────────────────────────────────────────────────────────────
    auto* titleBar   = new QHBoxLayout;
    auto* titleLabel = new QLabel(QStringLiteral("HashCalc"));
    QFont tf         = titleLabel->font();
    tf.setPointSize(14);
    tf.setBold(true);
    titleLabel->setFont(tf);

    m_themeButton = new QPushButton(QStringLiteral("☀"));
    m_themeButton->setFixedSize(28, 28);
    m_themeButton->setToolTip(QStringLiteral("Basculer clair / sombre"));
    titleBar->addWidget(titleLabel);
    titleBar->addStretch();
    titleBar->addWidget(m_themeButton);
    mainLayout->addLayout(titleBar);

    auto* sep = new QFrame;
    sep->setFrameShape(QFrame::HLine);
    mainLayout->addWidget(sep);

    // ── Algorithm selector ────────────────────────────────────────────────────
    auto* algoRow = new QHBoxLayout;
    algoRow->addWidget(new QLabel(QStringLiteral("Algorithme :")));
    m_algoCombo = new QComboBox;
    m_algoCombo->setFixedWidth(200);
    const int count = hashcalc_algorithm_count();
    for (int i = 0; i < count; ++i)
        m_algoCombo->addItem(QString::fromUtf8(hashcalc_algorithm_name(i)));
    algoRow->addWidget(m_algoCombo);
    algoRow->addStretch();
    mainLayout->addLayout(algoRow);

    // ── Drop zone ─────────────────────────────────────────────────────────────
    m_dropZone = new DropZone;
    mainLayout->addWidget(m_dropZone);

    // ── File path (small, dimmed) ─────────────────────────────────────────────
    m_filePathLabel = new QLabel;
    m_filePathLabel->setWordWrap(true);
    QFont small = m_filePathLabel->font();
    small.setPointSize(qMax(7, small.pointSize() - 1));
    m_filePathLabel->setFont(small);
    mainLayout->addWidget(m_filePathLabel);

    // ── Progress bar + Stop button ────────────────────────────────────────────
    m_progressWidget = new QWidget;
    auto* progressRow = new QHBoxLayout(m_progressWidget);
    progressRow->setContentsMargins(0, 0, 0, 0);
    m_progressBar = new QProgressBar;
    m_progressBar->setRange(0, 0);
    m_progressBar->setTextVisible(false);
    m_progressBar->setFixedHeight(14);
    m_stopButton = new QPushButton(QStringLiteral("⏹ Stop"));
    m_stopButton->setFixedWidth(72);
    progressRow->addWidget(m_progressBar);
    progressRow->addWidget(m_stopButton);
    m_progressWidget->setVisible(false);
    mainLayout->addWidget(m_progressWidget);

    // ── Result row ────────────────────────────────────────────────────────────
    m_resultLabel = new QLabel(QStringLiteral("Résultat :"));
    m_resultLabel->setVisible(false);
    mainLayout->addWidget(m_resultLabel);

    auto* resultRow = new QHBoxLayout;
    m_resultEdit = new QLineEdit;
    m_resultEdit->setReadOnly(true);
    m_resultEdit->setPlaceholderText(QStringLiteral("Le hash apparaîtra ici…"));
    m_resultEdit->setFont(QFontDatabase::systemFont(QFontDatabase::FixedFont));
    m_copyButton = new QPushButton(QStringLiteral("Copier"));
    m_copyButton->setEnabled(false);
    m_copyButton->setFixedWidth(80);
    resultRow->addWidget(m_resultEdit);
    resultRow->addWidget(m_copyButton);
    mainLayout->addLayout(resultRow);

    // ── Error label ───────────────────────────────────────────────────────────
    m_errorLabel = new QLabel;
    m_errorLabel->setWordWrap(true);
    m_errorLabel->setVisible(false);
    mainLayout->addWidget(m_errorLabel);

    mainLayout->addStretch();

    // ── Poll timer (reads progress from main thread, like egui's request_repaint loop) ──
    m_pollTimer = new QTimer(this);
    m_pollTimer->setInterval(80);
    connect(m_pollTimer, &QTimer::timeout, this, [this] {
        if (!m_bytesDone) return;
        const uint64_t total = m_fileSize ->load(std::memory_order_relaxed);
        const uint64_t done  = m_bytesDone->load(std::memory_order_relaxed);
        if (total > 0) {
            m_progressBar->setRange(0, 1000);
            m_progressBar->setValue(
                static_cast<int>(done > total ? 1000 : done * 1000 / total));
        }
    });

    // ── Signals ───────────────────────────────────────────────────────────────
    connect(m_dropZone, &DropZone::fileDropped, this, &MainWindow::onFileSelected);
    connect(m_dropZone, &DropZone::clicked, this, [this] {
        const QString path = QFileDialog::getOpenFileName(
            this, QStringLiteral("Sélectionner un fichier"));
        if (!path.isEmpty())
            onFileSelected(path);
    });
    connect(m_algoCombo, QOverload<int>::of(&QComboBox::currentIndexChanged),
            this, [this](int) { if (!m_filePath.isEmpty()) startHash(); });
    connect(m_copyButton,  &QPushButton::clicked, this, &MainWindow::onCopyClicked);
    connect(m_themeButton, &QPushButton::clicked, this, &MainWindow::onThemeToggled);
    connect(m_stopButton,  &QPushButton::clicked, this, [this] {
        if (m_cancelled)
            m_cancelled->store(1, std::memory_order_relaxed);
        ++m_currentGen;          // discard the in-flight result even if it arrives before cancellation
        m_pollTimer->stop();
        m_progressWidget->setVisible(false);
    });

    applyTheme();
}

void MainWindow::onFileSelected(const QString& path) {
    m_filePath = path;
    m_dropZone->setFileName(QFileInfo(path).fileName());
    m_filePathLabel->setText(path);
    startHash();
}

// Direct translation of egui's start_hash():
//   - New shared_ptr<atomic> each call  →  like Arc::new(AtomicU64::new(0))
//   - Old thread still runs but writes into its own (now-orphaned) shared_ptr copy
//   - Old result is ignored via generation check
//   - This function returns immediately; progress is polled by m_pollTimer
void MainWindow::startHash() {
    // ── Reset UI ──────────────────────────────────────────────────────────────
    m_pollTimer->stop();
    m_resultEdit->clear();
    m_resultLabel->setVisible(false);
    m_copyButton->setEnabled(false);
    m_errorLabel->setVisible(false);
    m_progressBar->setRange(0, 0); // indeterminate until file size is known
    m_progressBar->setValue(0);
    m_progressWidget->setVisible(true);

    // ── Cancel any in-flight computation ─────────────────────────────────────
    if (m_cancelled)
        m_cancelled->store(1, std::memory_order_relaxed);

    // ── New per-computation state (like Arc::new in egui) ─────────────────────
    m_bytesDone = std::make_shared<std::atomic<uint64_t>>(0);
    m_fileSize  = std::make_shared<std::atomic<uint64_t>>(0);
    m_cancelled = std::make_shared<std::atomic<uint8_t>>(0);

    const quint64 myGen  = ++m_currentGen;
    const QString path   = m_filePath;
    const int     algo   = m_algoCombo->currentIndex();

    // Each thread holds its own shared_ptr copies; the old thread's cancelled
    // flag is already set to 1 above, so it will stop at its next chunk boundary.
    auto bytesDone = m_bytesDone;
    auto fileSize  = m_fileSize;
    auto cancelled = m_cancelled;

    m_pollTimer->start();

    // ── Spawn and detach (like std::thread::spawn in egui) ───────────────────
    std::thread([this, path, algo, myGen,
                 bytesDone = std::move(bytesDone),
                 fileSize  = std::move(fileSize),
                 cancelled = std::move(cancelled)]() mutable
    {
        const QByteArray pathBytes = path.toLocal8Bit();

        char* result = hashcalc_hash_file_progress(
            pathBytes.constData(),
            algo,
            reinterpret_cast<uint64_t*>(bytesDone.get()),
            reinterpret_cast<uint64_t*>(fileSize.get()),
            reinterpret_cast<const uint8_t*>(cancelled.get()));

        const bool    ok   = (result != nullptr);
        const QString hash = ok ? QString::fromUtf8(result) : QString{};
        hashcalc_free_string(result);

        // Post back to main thread. Qt discards this if MainWindow is gone.
        QMetaObject::invokeMethod(this,
            [this, myGen, ok, hash]() {
                onComputeDone(ok, hash, myGen);
            },
            Qt::QueuedConnection);
    }).detach();
}

void MainWindow::onComputeDone(bool ok, const QString& result, quint64 gen) {
    if (gen != m_currentGen)
        return; // superseded request — discard silently

    m_pollTimer->stop();

    if (ok) {
        m_progressBar->setRange(0, 1000);
        m_progressBar->setValue(1000);
        QTimer::singleShot(300, this, [this] { m_progressWidget->setVisible(false); });
        m_resultLabel->setVisible(true);
        m_resultEdit->setText(result);
        m_copyButton->setEnabled(true);
    } else {
        m_progressWidget->setVisible(false);
        m_errorLabel->setText(
            QStringLiteral("Erreur : impossible de calculer le hash (E/S ou chemin invalide)"));
        m_errorLabel->setVisible(true);
    }
}

void MainWindow::onCopyClicked() {
    QApplication::clipboard()->setText(m_resultEdit->text());
}

void MainWindow::onThemeToggled() {
    m_darkMode = !m_darkMode;
    applyTheme();
}

void MainWindow::applyTheme() {
    if (m_darkMode) {
        QPalette dark;
        dark.setColor(QPalette::Window,         QColor(45,  45,  45));
        dark.setColor(QPalette::WindowText,      Qt::white);
        dark.setColor(QPalette::Base,            QColor(30,  30,  30));
        dark.setColor(QPalette::AlternateBase,   QColor(53,  53,  53));
        dark.setColor(QPalette::ToolTipBase,     Qt::white);
        dark.setColor(QPalette::ToolTipText,     Qt::white);
        dark.setColor(QPalette::Text,            Qt::white);
        dark.setColor(QPalette::Button,          QColor(53,  53,  53));
        dark.setColor(QPalette::ButtonText,      Qt::white);
        dark.setColor(QPalette::BrightText,      Qt::red);
        dark.setColor(QPalette::Highlight,       QColor(42, 130, 218));
        dark.setColor(QPalette::HighlightedText, Qt::black);
        dark.setColor(QPalette::Mid,             QColor(100, 100, 100));
        dark.setColor(QPalette::Shadow,          QColor(20,  20,  20));
        qApp->setPalette(dark);
        m_themeButton->setText(QStringLiteral("☀"));
        m_errorLabel->setStyleSheet(QStringLiteral("color: #ff6b6b;"));
    } else {
        qApp->setPalette(qApp->style()->standardPalette());
        m_themeButton->setText(QStringLiteral("\U0001F319")); // 🌙
        m_errorLabel->setStyleSheet(QStringLiteral("color: #cc0000;"));
    }
}
