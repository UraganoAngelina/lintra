//! Called only from the owned-window synchronous Tauri command on the GUI thread.
use crate::ipc::ErrorCode;
use std::path::PathBuf;
use tauri::WebviewWindow;

#[cfg(target_os = "linux")]
pub(crate) fn choose(window: &WebviewWindow) -> Result<Option<PathBuf>, ErrorCode> {
    use gtk::prelude::*;
    if !gtk::is_initialized_main_thread() {
        return Err(ErrorCode::TechnicalFailure);
    }
    let parent = window
        .gtk_window()
        .map_err(|_| ErrorCode::TechnicalFailure)?;
    choose_gtk(parent.upcast_ref())
}

#[cfg(target_os = "linux")]
fn choose_gtk(parent: &gtk::Window) -> Result<Option<PathBuf>, ErrorCode> {
    use gtk::prelude::*;
    let dialog = gtk::FileChooserDialog::with_buttons(
        Some("Seleziona il file di input"),
        Some(parent),
        gtk::FileChooserAction::Open,
        &[
            ("Annulla", gtk::ResponseType::Cancel),
            ("Seleziona", gtk::ResponseType::Accept),
        ],
    );
    dialog.set_modal(true);
    dialog.set_select_multiple(false);
    dialog.set_local_only(true);
    dialog.set_preview_widget_active(false);
    let response = dialog.run();
    let path = if response == gtk::ResponseType::Accept {
        dialog
            .filename()
            .map(Some)
            .ok_or(ErrorCode::TechnicalFailure)
    } else {
        Ok(None)
    };
    dialog.close();
    path
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    use gtk::prelude::*;
    use std::{cell::Cell, rc::Rc, time::Duration};

    #[test]
    #[ignore = "Requires an owned native GTK display; run selectively in foreground"]
    fn real_gtk_picker_preserves_the_native_selected_path() {
        gtk::init().unwrap();
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("synthetic input = file.txt");
        std::fs::write(&path, b"001A  \n").unwrap();
        let parent = gtk::Window::new(gtk::WindowType::Toplevel);
        parent.set_title("Lintra · Prova scelta input sintetico");
        parent.show();
        let selected = Rc::new(Cell::new(false));
        let callback_selected = selected.clone();
        let callback_path = path.clone();
        let callback_parent = parent.clone();
        let accept = gtk::glib::timeout_add_local(Duration::from_millis(250), move || {
            for window in gtk::Window::list_toplevels() {
                if let Ok(dialog) = window.downcast::<gtk::FileChooserDialog>() {
                    if dialog.transient_for().as_ref() == Some(&callback_parent) {
                        if !callback_selected.get() {
                            assert!(dialog.set_filename(&callback_path));
                            callback_selected.set(true);
                        }
                        // GTK resolves the selected folder/file asynchronously.
                        if dialog.filename().as_ref() == Some(&callback_path) {
                            dialog.response(gtk::ResponseType::Accept);
                        }
                    }
                }
            }
            gtk::glib::ControlFlow::Continue
        });
        let timeout_parent = parent.clone();
        let timeout = gtk::glib::timeout_add_local(Duration::from_secs(10), move || {
            for window in gtk::Window::list_toplevels() {
                if let Ok(dialog) = window.downcast::<gtk::FileChooserDialog>() {
                    if dialog.transient_for().as_ref() == Some(&timeout_parent) {
                        dialog.response(gtk::ResponseType::Cancel);
                    }
                }
            }
            gtk::glib::ControlFlow::Continue
        });
        let result = choose_gtk(&parent);
        accept.remove();
        timeout.remove();
        parent.close();
        assert!(selected.get());
        assert_eq!(result.unwrap(), Some(path.clone()));
        assert_eq!(std::fs::read(path).unwrap(), b"001A  \n");
    }
}

#[cfg(target_os = "windows")]
pub(crate) fn choose(window: &WebviewWindow) -> Result<Option<PathBuf>, ErrorCode> {
    use std::{ffi::OsString, os::windows::ffi::OsStringExt};
    use windows::{
        Win32::{
            Foundation::ERROR_CANCELLED,
            System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance, CoTaskMemFree},
            UI::Shell::*,
        },
        core::{HRESULT, w},
    };
    let owner = window.hwnd().map_err(|_| ErrorCode::TechnicalFailure)?;
    // SAFETY: the owning Tauri/WebView GUI thread already has its COM apartment.
    // COM objects stay on this thread; the API-owned UTF-16 buffer is copied
    // to a native OsString and freed with CoTaskMemFree before returning.
    let result: windows::core::Result<Option<PathBuf>> = unsafe {
        (|| {
            let dialog: IFileOpenDialog =
                CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)?;
            dialog.SetOptions(
                FOS_FORCEFILESYSTEM
                    | FOS_FILEMUSTEXIST
                    | FOS_PATHMUSTEXIST
                    | FOS_NOCHANGEDIR
                    | FOS_DONTADDTORECENT,
            )?;
            dialog.SetTitle(w!("Seleziona il file di input"))?;
            if let Err(error) = dialog.Show(Some(owner)) {
                if error.code() == HRESULT::from_win32(ERROR_CANCELLED.0) {
                    return Ok(None);
                }
                return Err(error);
            }
            let name = dialog.GetResult()?.GetDisplayName(SIGDN_FILESYSPATH)?;
            let path = PathBuf::from(OsString::from_wide(name.as_wide()));
            CoTaskMemFree(Some(name.0.cast()));
            Ok(Some(path))
        })()
    };
    result.map_err(|_| ErrorCode::TechnicalFailure)
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub(crate) fn choose(_: &WebviewWindow) -> Result<Option<PathBuf>, ErrorCode> {
    Err(ErrorCode::TechnicalFailure)
}
