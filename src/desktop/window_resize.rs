use std::ffi::c_void;

use i_slint_backend_qt::QtWidgetAccessor;
use slint::ComponentHandle;

use crate::DesktopWindow;

unsafe extern "C" {
    fn velox_start_window_resize(widget: *mut c_void, horizontal: i32, vertical: i32);
}

pub(super) fn wire(window: &DesktopWindow) {
    let weak = window.as_weak();
    window.on_resize_from_edge(move |horizontal, vertical| {
        if let Some(window) = weak.upgrade()
            && let Some(widget) = window.window().qt_widget_ptr()
        {
            // Slint owns the QWidget and keeps it alive on this UI thread while
            // the native call handles the current mouse press.
            unsafe { velox_start_window_resize(widget.as_ptr().cast(), horizontal, vertical) };
        }
    });
}
