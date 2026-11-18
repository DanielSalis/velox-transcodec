#include <QtGui/QWindow>
#include <QtWidgets/QWidget>

extern "C" void velox_start_window_resize(void *widget, int horizontal, int vertical)
{
    auto *window = static_cast<QWidget *>(widget)->windowHandle();
    Qt::Edges edges;
    if (horizontal != 0)
        edges |= horizontal < 0 ? Qt::LeftEdge : Qt::RightEdge;
    if (vertical != 0)
        edges |= vertical < 0 ? Qt::TopEdge : Qt::BottomEdge;
    if (window && edges)
        window->startSystemResize(edges);
}
