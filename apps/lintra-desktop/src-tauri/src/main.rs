#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod security;
#[cfg(target_os = "windows")]
mod windows_request_guard;

use security::local_asset_origin;
#[cfg(target_os = "windows")]
use security::local_resource_origin;
use tauri::{Manager, WebviewWindowBuilder, webview::NewWindowResponse};

#[cfg(target_os = "windows")]
#[derive(Default)]
struct GuardCounters {
    resource_guard_ready: std::sync::atomic::AtomicBool,
    resource_guard_failed: std::sync::atomic::AtomicBool,
    web_resource_denied: std::sync::atomic::AtomicU32,
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
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
                .on_page_load(|window, payload| {
                    if !local_asset_origin(payload.url()) {
                        window.app_handle().exit(1);
                    }
                });
            #[cfg(target_os = "windows")]
            let window = window.on_web_resource_request(|_, response| {
                security::restrict_document_connections(response.headers_mut());
            });
            let built = window.build()?;
            #[cfg(target_os = "windows")]
            windows_request_guard::install(&built, std::sync::Arc::default())?;
            #[cfg(not(target_os = "windows"))]
            let _ = built;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("cannot run the desktop adapter");
}
