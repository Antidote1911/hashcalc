#pragma once
#include <QFrame>
#include <QString>

class DropZone : public QFrame {
    Q_OBJECT
public:
    explicit DropZone(QWidget* parent = nullptr);
    void setFileName(const QString& name);

signals:
    void fileDropped(const QString& path);
    void clicked();

protected:
    void dragEnterEvent(QDragEnterEvent* e) override;
    void dragLeaveEvent(QDragLeaveEvent* e) override;
    void dropEvent(QDropEvent* e) override;
    void mousePressEvent(QMouseEvent* e) override;
    void paintEvent(QPaintEvent* e) override;

private:
    bool    m_hovering{false};
    QString m_fileName;
};
