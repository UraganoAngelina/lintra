#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod budget;
mod configuration;
mod exports;
mod ipc;
mod jobs;
mod native_picker;
mod progress;
mod results;
mod security;
mod state;
// Applied on Windows only; the pure fitting logic is tested on every OS.
#[cfg(any(target_os = "windows", test))]
mod window_fit;
#[cfg(target_os = "windows")]
mod windows_request_guard;

use security::local_asset_origin;
#[cfg(target_os = "windows")]
use security::local_resource_origin;
use state::AppState;
#[cfg(target_os = "windows")]
use state::GuardCounters;
use tauri::{Manager, State, WebviewWindow, WebviewWindowBuilder, webview::NewWindowResponse};

/// Native dialogs owned by the calling window. Each answer obtained while
/// the application is closing is stale: a late dialog cannot resurrect a
/// selection or a confirmation of a closed instance.
struct WindowNative<'a> {
    window: &'a WebviewWindow,
    state: &'a AppState,
}
impl WindowNative<'_> {
    fn live<T>(&self, answer: T) -> Result<T, ipc::ErrorCode> {
        if self.state.closed() {
            return Err(ipc::ErrorCode::StaleContext);
        }
        Ok(answer)
    }
}
impl state::Native for WindowNative<'_> {
    fn choose_input(&self) -> Result<Option<std::path::PathBuf>, ipc::ErrorCode> {
        self.live(native_picker::choose(self.window)?)
    }
}
impl exports::ExportDialogs for WindowNative<'_> {
    fn save(&self, extension: &str) -> Result<Option<std::path::PathBuf>, ipc::ErrorCode> {
        self.live(native_picker::save(self.window, extension)?)
    }
    fn confirm(&self, title: &str, message: &str) -> Result<bool, ipc::ErrorCode> {
        self.live(native_picker::confirm(self.window, title, message)?)
    }
}

#[tauri::command]
fn lintra_ipc(
    window: WebviewWindow,
    request: tauri::ipc::Request<'_>,
    state: State<'_, AppState>,
) -> tauri::ipc::Response {
    let bytes = match window.url() {
        Ok(origin) => {
            let raw = match request.body() {
                tauri::ipc::InvokeBody::Raw(bytes) => Some(bytes.as_slice()),
                tauri::ipc::InvokeBody::Json(_) => None,
            };
            state.handle(
                window.label(),
                &origin,
                raw,
                &WindowNative {
                    window: &window,
                    state: &state,
                },
            )
        }
        Err(_) => ipc::error(None, ipc::ErrorCode::TechnicalFailure),
    };
    if state.smoke && state.ready() {
        println!("LINTRA_DESKTOP_READY");
        window.app_handle().exit(0);
    }
    tauri::ipc::Response::new(bytes)
}

fn main() {
    let shell_smoke = std::env::args()
        .skip(1)
        .any(|arg| arg == "--shell-smoke-test");
    tauri::Builder::default()
        .manage(AppState::new(
            std::env::args().skip(1).any(|arg| arg == "--smoke-test"),
        ))
        .setup(move |app| {
            // D09: per-user configuration directory for resource-budget.json.
            // Not resolvable means no budget can be configured (documented).
            if let Ok(directory) = app.path().app_config_dir() {
                let _ = app.state::<AppState>().budget_directory.set(directory);
            }
            let config = app
                .config()
                .app
                .windows
                .iter()
                .find(|window| window.label == "main")
                .ok_or("missing main window configuration")?;
            let window = WebviewWindowBuilder::from_config(app, config)?
                .on_navigation(local_asset_origin)
                .on_new_window(|_, _| NewWindowResponse::Deny)
                .on_page_load(move |window, payload| {
                    if !local_asset_origin(payload.url()) {
                        window.app_handle().exit(1);
                    } else if shell_smoke
                        && payload.event() == tauri::webview::PageLoadEvent::Finished
                    {
                        println!("LINTRA_DESKTOP_SHELL_READY");
                        window.app_handle().exit(0);
                    }
                });
            #[cfg(target_os = "windows")]
            // Built hidden and shown only after the fit below: when the fit
            // ran on a visible window, the first show could restore the
            // pre-fit placement and leave it past the work area (T-MIG-P5-08,
            // seen on the Windows 11 VM in 2 to 5 launches out of 32).
            let window = window
                .on_web_resource_request(|_, response| {
                    security::restrict_document_connections(response.headers_mut());
                })
                .visible(false);
            let built = window.build()?;
            // Windows lets the app place its window, so it must keep it inside
            // the work area (Tauri brief). Wayland compositors own placement and
            // apply size requests asynchronously, so no fit is attempted there.
            // Best effort: an unmeasurable monitor keeps the configured size.
            #[cfg(target_os = "windows")]
            window_fit::fit_window(
                &built,
                (
                    config.min_width.unwrap_or(0.0),
                    config.min_height.unwrap_or(0.0),
                ),
            )
            .ok();
            #[cfg(target_os = "windows")]
            built.show()?;
            #[cfg(target_os = "windows")]
            windows_request_guard::install(&built, app.state::<AppState>().guards.clone())?;
            #[cfg(not(target_os = "windows"))]
            let _ = built;
            Ok(())
        })
        .on_window_event(|window, event| {
            if matches!(
                event,
                tauri::WindowEvent::CloseRequested { .. } | tauri::WindowEvent::Destroyed
            ) {
                window.state::<AppState>().close();
            }
        })
        .invoke_handler(tauri::generate_handler![lintra_ipc])
        .build(tauri::generate_context!())
        .expect("cannot build the desktop adapter")
        .run(|app, event| {
            if matches!(event, tauri::RunEvent::Exit) {
                app.state::<AppState>().shutdown();
            }
        });
}
