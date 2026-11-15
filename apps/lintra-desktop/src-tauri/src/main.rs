#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod configuration;
mod ipc;
mod native_picker;
mod security;
mod state;
#[cfg(target_os = "windows")]
mod windows_request_guard;

use security::local_asset_origin;
#[cfg(target_os = "windows")]
use security::local_resource_origin;
use state::AppState;
#[cfg(target_os = "windows")]
use state::GuardCounters;
use tauri::{Manager, State, WebviewWindow, WebviewWindowBuilder, webview::NewWindowResponse};

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
            state.handle(window.label(), &origin, raw, &|| {
                let selected = native_picker::choose(&window)?;
                if state.closed() {
                    return Err(ipc::ErrorCode::StaleContext);
                }
                Ok(selected)
            })
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
            let window = window.on_web_resource_request(|_, response| {
                security::restrict_document_connections(response.headers_mut());
            });
            let built = window.build()?;
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
        .run(tauri::generate_context!())
        .expect("cannot run the desktop adapter");
}
