#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};
use tauri::{
    AppHandle, State, Url, WebviewWindow, WebviewWindowBuilder, webview::NewWindowResponse,
};

#[derive(Default)]
struct GuardCounters {
    navigation_denied: AtomicU32,
    new_window_denied: AtomicU32,
}

struct ProbeOptions {
    exit_after_render: bool,
    security_test: bool,
    guards: Arc<GuardCounters>,
}

/// Fixed-size synthetic results only: no URL, path, record or arbitrary map
/// can enter this acknowledgment. The renderer tests the deployed CSP/ACL;
/// native counters independently require both navigation guards to execute.
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SecurityReport {
    text_only: bool,
    inline_script_blocked: bool,
    remote_script_blocked: bool,
    remote_style_blocked: bool,
    remote_font_blocked: bool,
    image_blocked: bool,
    frame_blocked: bool,
    object_blocked: bool,
    fetch_blocked: bool,
    websocket_blocked: bool,
    eventsource_blocked: bool,
    beacon_blocked: bool,
    form_blocked: bool,
    native_navigation_blocked: bool,
    native_popup_blocked: bool,
    valid_invoke: bool,
    forbidden_invokes: bool,
    invalid_reports: bool,
    csp_events: u32,
}

impl SecurityReport {
    fn passed(&self) -> bool {
        self.text_only
            && self.inline_script_blocked
            && self.remote_script_blocked
            && self.remote_style_blocked
            && self.remote_font_blocked
            && self.image_blocked
            && self.frame_blocked
            && self.object_blocked
            && self.fetch_blocked
            && self.websocket_blocked
            && self.eventsource_blocked
            && self.beacon_blocked
            && self.form_blocked
            && self.native_navigation_blocked
            && self.native_popup_blocked
            && self.valid_invoke
            && self.forbidden_invokes
            && self.invalid_reports
            && (10..=128).contains(&self.csp_events)
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RenderReport {
    protocol: u8,
    width: u32,
    height: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    security: Option<SecurityReport>,
}

/// Match origins, not URL prefixes. Credentials, nonstandard ports and lookalike
/// hosts are rejected. These are the exact pinned Tauri asset origins; neither
/// filesystem URLs nor network origins acquire authority from a window label.
fn local_asset_origin(url: &Url) -> bool {
    url.username().is_empty()
        && url.password().is_none()
        && url.port().is_none()
        && matches!(
            (url.scheme(), url.host_str()),
            ("tauri", Some("localhost")) | ("http", Some("tauri.localhost"))
        )
}

fn valid_render_bounds(report: &RenderReport) -> bool {
    report.protocol == 1
        && (1..=32768).contains(&report.width)
        && (1..=32768).contains(&report.height)
}

/// Check the actual caller as well as the capability. No path, file content,
/// schema, keys or arbitrary strings cross this probe's IPC boundary.
fn authorize(window: &WebviewWindow) -> Result<(), &'static str> {
    if window.label() != "probe" {
        return Err("unexpected probe window");
    }
    if !local_asset_origin(&window.url().map_err(|_| "cannot read probe origin")?) {
        return Err("unexpected probe origin");
    }
    Ok(())
}

#[tauri::command]
fn runtime_probe(window: WebviewWindow) -> Result<&'static str, &'static str> {
    authorize(&window)?;
    Ok("lintra-runtime-v1")
}

#[tauri::command]
fn render_ack(
    window: WebviewWindow,
    app: AppHandle,
    options: State<'_, ProbeOptions>,
    report: RenderReport,
) -> Result<(), &'static str> {
    authorize(&window)?;
    // Bounds constrain the synthetic proof, not future product DTO limits.
    if !valid_render_bounds(&report) {
        return Err("invalid runtime render acknowledgment");
    }
    if options.security_test {
        let Some(security) = report.security.as_ref() else {
            // A bounded handshake for the foreground OS-input harness. Some
            // browsers reject scripted popups before our callback; a trusted
            // click is needed to exercise the native denial itself.
            println!("LINTRA_RUNTIME_SECURITY_AWAIT_POPUP");
            return Err("security report required");
        };
        let navigation = options.guards.navigation_denied.load(Ordering::Relaxed);
        let popups = options.guards.new_window_denied.load(Ordering::Relaxed);
        println!(
            "LINTRA_RUNTIME_SECURITY_REPORT {}",
            serde_json::to_string(security).map_err(|_| "cannot encode security acknowledgment")?
        );
        if !security.passed() || navigation == 0 || popups == 0 {
            app.exit(1);
            return Err("security probe failed");
        }
        println!(
            "LINTRA_RUNTIME_SECURITY_ACK {{\"navigation_denied\":{navigation},\"new_window_denied\":{popups}}}"
        );
    } else if report.security.is_some() {
        return Err("security report outside qualification mode");
    }
    println!(
        "LINTRA_RUNTIME_RENDER_ACK {}",
        serde_json::to_string(&report).map_err(|_| "cannot encode runtime acknowledgment")?
    );
    if options.exit_after_render || options.security_test {
        app.exit(0);
    }
    Ok(())
}

fn main() {
    // Smoke mode exits only after a real React render and Rust round trip.
    // Normal mode remains open for native window inspection and capture.
    let exit_after_render = std::env::args().skip(1).any(|arg| arg == "--smoke-test");
    let security_test = std::env::args().skip(1).any(|arg| arg == "--security-test");
    let guards = Arc::new(GuardCounters::default());
    tauri::Builder::default()
        .manage(ProbeOptions {
            exit_after_render,
            security_test,
            guards: guards.clone(),
        })
        .setup(move |app| {
            let config = app
                .config()
                .app
                .windows
                .iter()
                .find(|config| config.label == "probe")
                .ok_or("missing probe window configuration")?;
            let navigation_guards = guards.clone();
            let popup_guards = guards.clone();
            // Disable automatic window creation in config so these callbacks
            // are attached before the very first document is loaded.
            let mut window = WebviewWindowBuilder::from_config(app, config)?
                .on_navigation(move |url| {
                    let allowed = local_asset_origin(url);
                    if !allowed {
                        navigation_guards
                            .navigation_denied
                            .fetch_add(1, Ordering::Relaxed);
                        println!("LINTRA_RUNTIME_NAVIGATION_DENIED");
                    }
                    allowed
                })
                .on_new_window(move |_url, _features| {
                    popup_guards
                        .new_window_denied
                        .fetch_add(1, Ordering::Relaxed);
                    println!("LINTRA_RUNTIME_NEW_WINDOW_DENIED");
                    NewWindowResponse::Deny
                });
            if security_test {
                window = window.initialization_script("window.__LINTRA_SECURITY_TEST__ = true;");
            }
            window.build()?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![runtime_probe, render_ack])
        .run(tauri::generate_context!())
        .expect("failed to run the isolated Tauri runtime probe");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_origins_reject_lookalikes_credentials_ports_and_other_protocols() {
        for url in [
            "tauri://localhost/",
            "tauri://localhost/index.html#/probe",
            "http://tauri.localhost/",
        ] {
            assert!(local_asset_origin(&Url::parse(url).unwrap()), "{url}");
        }
        for url in [
            "https://tauri.localhost/",
            "http://tauri.localhost.evil/",
            "http://tauri.localhost:1234/",
            "http://user@tauri.localhost/",
            "http://user:secret@tauri.localhost/",
            "tauri://elsewhere/",
            "file:///etc/passwd",
            "data:text/html,hello",
            "about:blank",
            "http://127.0.0.1/",
        ] {
            assert!(!local_asset_origin(&Url::parse(url).unwrap()), "{url}");
        }
    }

    #[test]
    fn render_bounds_and_wire_shape_fail_closed() {
        for (protocol, width, height, valid) in [
            (1, 1, 1, true),
            (1, 32768, 32768, true),
            (0, 900, 600, false),
            (2, 900, 600, false),
            (1, 0, 600, false),
            (1, 900, 0, false),
            (1, 32769, 600, false),
            (1, 900, 32769, false),
        ] {
            assert_eq!(
                valid_render_bounds(&RenderReport {
                    protocol,
                    width,
                    height,
                    security: None
                }),
                valid
            );
        }
        for json in [
            r#"{"protocol":1,"width":900,"height":600,"extra":true}"#,
            r#"{"protocol":1,"width":"900","height":600}"#,
            r#"{"protocol":1,"width":4294967296,"height":600}"#,
            "null",
        ] {
            assert!(serde_json::from_str::<RenderReport>(json).is_err());
        }
    }
}
