#include "DropZone.h"

#include <QDragEnterEvent>
#include <QDropEvent>
#include <QMimeData>
#include <QMouseEvent>
#include <QPainter>
#include <QPainterPath>
#include <QPen>
#include <QUrl>

DropZone::DropZone(QWidget* parent) : QFrame(parent) {
    setAcceptDrops(true);
    setMinimumHeight(90);
    setCursor(Qt::PointingHandCursor);
}

void DropZone::setFileName(const QString& name) {
    m_fileName = name;
    update();
}

void DropZone::dragEnterEvent(QDragEnterEvent* e) {
    if (e->mimeData()->hasUrls()) {
        auto urls = e->mimeData()->urls();
        if (!urls.isEmpty() && urls.first().isLocalFile()) {
            m_hovering = true;
            update();
            e->acceptProposedAction();
            return;
        }
    }
    e->ignore();
}

void DropZone::dragLeaveEvent(QDragLeaveEvent*) {
    m_hovering = false;
    update();
}

void DropZone::dropEvent(QDropEvent* e) {
    m_hovering = false;
    if (e->mimeData()->hasUrls()) {
        auto urls = e->mimeData()->urls();
        if (!urls.isEmpty()) {
            QString path = urls.first().toLocalFile();
            if (!path.isEmpty()) {
                e->acceptProposedAction();
                emit fileDropped(path);
                return;
            }
        }
    }
    e->ignore();
    update();
}

void DropZone::mousePressEvent(QMouseEvent* e) {
    if (e->button() == Qt::LeftButton)
        emit clicked();
}

void DropZone::paintEvent(QPaintEvent*) {
    QPainter p(this);
    p.setRenderHint(QPainter::Antialiasing);

    const QRectF r = QRectF(rect()).adjusted(1.0, 1.0, -1.0, -1.0);
    const qreal  radius = 8.0;

    // Background fill on hover
    if (m_hovering) {
        p.setBrush(QColor(80, 200, 80, 30));
    } else {
        p.setBrush(Qt::NoBrush);
    }

    // Border
    QPen pen;
    pen.setWidthF(m_hovering ? 2.0 : 1.5);
    pen.setStyle(Qt::DashLine);
    pen.setColor(m_hovering ? QColor(80, 200, 80)
                            : palette().color(QPalette::Mid));
    p.setPen(pen);
    p.drawRoundedRect(r, radius, radius);

    // Label
    QString label = m_hovering          ? u8"\U0001F4C2  Déposer le fichier…"
                  : m_fileName.isEmpty() ? u8"\U0001F4C2  Déposer un fichier ici ou cliquer pour parcourir"
                                         : u8"\U0001F4C4  " + m_fileName;

    p.setPen(palette().color(QPalette::Text));
    p.drawText(rect(), Qt::AlignCenter, label);
}
