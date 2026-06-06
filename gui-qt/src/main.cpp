#include <QApplication>
#include "MainWindow.h"

int main(int argc, char* argv[]) {
    QApplication app(argc, argv);
    app.setApplicationName(QStringLiteral("HashCalc"));
    app.setApplicationVersion(QStringLiteral("0.4.1"));

    MainWindow window;
    window.show();

    return app.exec();
}
