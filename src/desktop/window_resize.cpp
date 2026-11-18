#include <QtGui/QWindow>
#include <QtWidgets/QWidget>

#ifdef Q_OS_MACOS
void velox_resize_cocoa(QWidget *widget, Qt::Edges edges);
#endif

extern "C" void velox_start_window_resize(void *widget, int horizontal, int vertical)
{
    auto *frame = static_cast<QWidget *>(widget);
    auto *window = frame->windowHandle();
    Qt::Edges edges;
    if (horizontal != 0)
        edges |= horizontal < 0 ? Qt::LeftEdge : Qt::RightEdge;
    if (vertical != 0)
        edges |= vertical < 0 ? Qt::TopEdge : Qt::BottomEdge;
    if (window && edges && !window->startSystemResize(edges)) {
#ifdef Q_OS_MACOS
        velox_resize_cocoa(frame, edges);
#endif
    }
}
