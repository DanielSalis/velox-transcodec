#include <QtCore/QFile>
#include <QtCore/QObject>
#include <QtGui/QCursor>
#include <QtGui/QFileOpenEvent>
#include <QtGui/QMouseEvent>
#include <QtWidgets/QApplication>
#include <QtWidgets/QWidget>

#include <algorithm>

namespace {
using OpenFile = void (*)(void *, const char *);

class FileOpenFilter : public QObject
{
public:
    FileOpenFilter(OpenFile open, void *context) : open(open), context(context) {}

    bool eventFilter(QObject *, QEvent *event) override
    {
        if (event->type() != QEvent::FileOpen)
            return false;

        auto *file = static_cast<QFileOpenEvent *>(event);
        auto url = file->url();
        auto path = url.isEmpty() ? file->file() : url.toLocalFile();
        if (path.isEmpty())
            return false;

        const auto encoded = QFile::encodeName(path);
        open(context, encoded.constData());
        return true;
    }

private:
    OpenFile open;
    void *context;
};

// Qt's Cocoa backend handles moving the window but has no system edge resize.
class ResizeGesture : public QObject
{
public:
    ResizeGesture(QWidget *window, Qt::Edges edges)
        : QObject(window), window(window), edges(edges), origin(QCursor::pos()),
          initial(window->geometry())
    {
        window->installEventFilter(this);
    }

    bool eventFilter(QObject *, QEvent *event) override
    {
        if (event->type() == QEvent::MouseMove) {
            auto *mouse = static_cast<QMouseEvent *>(event);
            if (mouse->buttons() & Qt::LeftButton) {
                resize(mouse->globalPosition().toPoint() - origin);
                return true;
            }
        } else if (event->type() != QEvent::MouseButtonRelease
                   && event->type() != QEvent::UngrabMouse
                   && event->type() != QEvent::WindowDeactivate
                   && event->type() != QEvent::Hide) {
            return false;
        }

        window->removeEventFilter(this);
        deleteLater();
        return false;
    }

private:
    void resize(QPoint delta)
    {
        QRect geometry = initial;
        const auto minimum = window->minimumSize();
        const auto maximum = window->maximumSize();
        if (edges & Qt::LeftEdge)
            geometry.setLeft(initial.right() + 1 -
                             std::clamp(initial.width() - delta.x(), minimum.width(), maximum.width()));
        if (edges & Qt::RightEdge)
            geometry.setWidth(std::clamp(initial.width() + delta.x(), minimum.width(), maximum.width()));
        if (edges & Qt::TopEdge)
            geometry.setTop(initial.bottom() + 1 -
                            std::clamp(initial.height() - delta.y(), minimum.height(), maximum.height()));
        if (edges & Qt::BottomEdge)
            geometry.setHeight(std::clamp(initial.height() + delta.y(), minimum.height(), maximum.height()));
        window->setGeometry(geometry);
    }

    QWidget *window;
    Qt::Edges edges;
    QPoint origin;
    QRect initial;
};
}

extern "C" void *velox_install_file_open(OpenFile open, void *context)
{
    auto *filter = new FileOpenFilter(open, context);
    qApp->installEventFilter(filter);
    return filter;
}

extern "C" void velox_remove_file_open(void *filter)
{
    delete static_cast<FileOpenFilter *>(filter);
}

void velox_resize_cocoa(QWidget *widget, Qt::Edges edges)
{
    new ResizeGesture(widget, edges);
}
