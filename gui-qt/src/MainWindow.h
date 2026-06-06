#pragma once
#include <QComboBox>
#include <QLabel>
#include <QLineEdit>
#include <QMainWindow>
#include <QProgressBar>
#include <QPushButton>
#include <QTimer>

#include <atomic>
#include <cstdint>
#include <memory>

#include "DropZone.h"

class MainWindow : public QMainWindow {
    Q_OBJECT
public:
    explicit MainWindow(QWidget* parent = nullptr);

private slots:
    void onFileSelected(const QString& path);
    void onComputeDone(bool ok, const QString& result, quint64 gen);
    void onCopyClicked();
    void onThemeToggled();

private:
    void startHash();
    void applyTheme();

    // ── per-computation state (shared with the detached thread, like Arc in Rust) ─
    std::shared_ptr<std::atomic<uint64_t>> m_bytesDone;
    std::shared_ptr<std::atomic<uint64_t>> m_fileSize;
    std::shared_ptr<std::atomic<uint8_t>>  m_cancelled;   // write 1 to stop old thread
    quint64 m_currentGen{0};

    QTimer*       m_pollTimer{nullptr};
    QWidget*      m_progressWidget{nullptr};

    DropZone*     m_dropZone{nullptr};
    QComboBox*    m_algoCombo{nullptr};
    QProgressBar* m_progressBar{nullptr};
    QLineEdit*    m_resultEdit{nullptr};
    QPushButton*  m_copyButton{nullptr};
    QPushButton*  m_stopButton{nullptr};
    QLabel*       m_filePathLabel{nullptr};
    QLabel*       m_resultLabel{nullptr};
    QLabel*       m_errorLabel{nullptr};
    QPushButton*  m_themeButton{nullptr};

    QString m_filePath;
    bool    m_darkMode{true};
};
