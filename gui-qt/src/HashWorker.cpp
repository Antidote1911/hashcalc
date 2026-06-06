#include "HashWorker.h"
#include <hashcalc_ffi.h>

HashWorker::HashWorker(QObject* parent) : QObject(parent) {}

void HashWorker::compute(const QString& path, int algoIndex, quint64 gen) {
    // Signal the main thread that THIS generation's computation is starting.
    // The main thread uses this to start the poll timer only for the current gen.
    emit started(gen);

    const QByteArray pathBytes = path.toLocal8Bit();
    uint64_t* pDone = reinterpret_cast<uint64_t*>(&m_bytesDone);
    uint64_t* pSize = reinterpret_cast<uint64_t*>(&m_fileSize);

    char* result = hashcalc_hash_file_progress(
        pathBytes.constData(), algoIndex, pDone, pSize);

    if (result) {
        emit finished(QString::fromUtf8(result), gen);
        hashcalc_free_string(result);
    } else {
        emit error(
            QStringLiteral("Impossible de calculer le hash (erreur I/O ou chemin invalide)"),
            gen);
    }
}
