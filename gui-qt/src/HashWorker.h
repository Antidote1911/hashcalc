#pragma once
#include <QObject>
#include <QString>
#include <atomic>
#include <cstdint>

class HashWorker : public QObject {
    Q_OBJECT
public:
    explicit HashWorker(QObject* parent = nullptr);

    // Safe to call from any thread (lock-free atomic load).
    uint64_t bytesDone() const { return m_bytesDone.load(std::memory_order_relaxed); }
    uint64_t fileSize()  const { return m_fileSize .load(std::memory_order_relaxed); }

public slots:
    // gen: generation token echoed in every outgoing signal so the main thread
    // can discard results that belong to superseded requests.
    void compute(const QString& path, int algoIndex, quint64 gen);

signals:
    void started (quint64 gen);
    void finished(const QString& hash, quint64 gen);
    void error   (const QString& message, quint64 gen);

private:
    std::atomic<uint64_t> m_bytesDone{0};
    std::atomic<uint64_t> m_fileSize {0};
};
