use std::{
    ffi::OsString,
    io,
    os::unix::ffi::OsStringExt,
    path::{Path, PathBuf},
};

use velox_transcode::media::MediaEngine;

pub(super) fn media_engine() -> io::Result<MediaEngine> {
    let executable = std::env::current_exe()?;
    let Some(contents) = executable
        .parent()
        .filter(|directory| directory.ends_with("Contents/MacOS"))
        .and_then(Path::parent)
    else {
        return Ok(MediaEngine::default());
    };
    let resources = contents.join("Resources");
    let tool = |name: &str| {
        let path = PathBuf::from(OsString::from_vec(std::fs::read(
            resources.join(format!("{name}.path")),
        )?));
        if !path.is_absolute() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("O bundle precisa indicar um caminho absoluto para {name}"),
            ));
        }
        Ok(path)
    };
    Ok(MediaEngine::new(tool("ffmpeg")?, tool("ffprobe")?))
}

#[cfg(velox_qt_resize)]
mod finder {
    use super::*;
    use std::ffi::{CStr, c_char, c_void};

    unsafe extern "C" {
        fn velox_install_file_open(
            open: extern "C" fn(*mut c_void, *const c_char),
            context: *mut c_void,
        ) -> *mut c_void;
        fn velox_remove_file_open(filter: *mut c_void);
    }

    pub(in crate::desktop) struct FileOpen<F> {
        filter: *mut c_void,
        _callback: Box<F>,
    }

    impl<F: Fn(PathBuf)> FileOpen<F> {
        pub(in crate::desktop) fn new(callback: F) -> Self {
            extern "C" fn open<F: Fn(PathBuf)>(context: *mut c_void, file: *const c_char) {
                // Qt invokes the callback on the UI thread with a live filter and path.
                let callback = unsafe { &*context.cast::<F>() };
                let bytes = unsafe { CStr::from_ptr(file) }.to_bytes().to_vec();
                callback(PathBuf::from(OsString::from_vec(bytes)));
            }

            let mut callback = Box::new(callback);
            // The Qt application already exists. The callback stays in the box until Drop
            // removes the Qt filter on the same thread.
            let filter =
                unsafe { velox_install_file_open(open::<F>, (&mut *callback as *mut F).cast()) };
            Self {
                filter,
                _callback: callback,
            }
        }
    }

    impl<F> Drop for FileOpen<F> {
        fn drop(&mut self) {
            // The filter is owned solely by this guard and still on the UI thread.
            unsafe { velox_remove_file_open(self.filter) };
        }
    }
}

#[cfg(velox_qt_resize)]
pub(super) use finder::FileOpen;
