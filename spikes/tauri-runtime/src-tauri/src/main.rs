#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};
use tauri::{
    AppHandle, Manager, State, Url, WebviewWindow, WebviewWindowBuilder, webview::NewWindowResponse,
};

#[cfg(target_os = "windows")]
mod windows_request_guard;

#[derive(Default)]
struct GuardCounters {
    navigation_denied: AtomicU32,
    new_window_denied: AtomicU32,
    web_resource_denied: AtomicU32,
    #[cfg(target_os = "windows")]
    resource_guard_ready: std::sync::atomic::AtomicBool,
    #[cfg(target_os = "windows")]
    resource_guard_failed: std::sync::atomic::AtomicBool,
}

struct ProbeOptions {
    exit_after_render: bool,
    security_test: bool,
    profile_test: bool,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    profile: Option<ProfileReport>,
}

/// Bounded, synthetic document measurements. Native window size/scale are
/// recorded independently; renderer-reported DPI alone is not an OS proof.
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ProfileReport {
    scale_milli: u32,
    body_font_milli: u32,
    title_font_milli: u32,
    normal_face_loaded: bool,
    semibold_face_loaded: bool,
    body_uses_plex: bool,
    title_uses_plex: bool,
    no_horizontal_overflow: bool,
    title_inside_viewport: bool,
}

impl ProfileReport {
    fn passed(&self) -> bool {
        (500..=8000).contains(&self.scale_milli)
            && self.body_font_milli == 14000
            && self.title_font_milli == 28000
            && self.normal_face_loaded
            && self.semibold_face_loaded
            && self.body_uses_plex
            && self.title_uses_plex
            && self.no_horizontal_overflow
            && self.title_inside_viewport
    }

    /// A CSS zoom must not masquerade as a native monitor scale. Compare both
    /// axes independently, allowing only two physical pixels of rounding.
    fn matches_native(&self, report: &RenderReport, physical: (u32, u32), scale: u32) -> bool {
        let size_matches = |physical: u32, logical: u32| {
            (i64::from(physical) * 1000 - i64::from(logical) * i64::from(scale)).abs() <= 2000
        };
        valid_render_bounds(report)
            && self.passed()
            && self.scale_milli.abs_diff(scale) <= 2
            && size_matches(physical.0, report.width)
            && size_matches(physical.1, report.height)
    }
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

#[cfg(any(target_os = "windows", test))]
fn local_resource_origin(url: &Url) -> bool {
    local_asset_origin(url)
        || (url.username().is_empty()
            && url.password().is_none()
            && url.port().is_none()
            && matches!(
                (url.scheme(), url.host_str()),
                ("http", Some("ipc.localhost")) | ("ipc", Some("localhost"))
            ))
}

fn ensure_request_guard(options: &ProbeOptions) -> Result<(), &'static str> {
    #[cfg(target_os = "windows")]
    if !options.guards.resource_guard_ready.load(Ordering::Relaxed)
        || options.guards.resource_guard_failed.load(Ordering::Relaxed)
    {
        return Err("native request guard unavailable");
    }
    #[cfg(not(target_os = "windows"))]
    let _ = options;
    Ok(())
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
fn runtime_probe(
    window: WebviewWindow,
    options: State<'_, ProbeOptions>,
) -> Result<&'static str, &'static str> {
    authorize(&window)?;
    ensure_request_guard(&options)?;
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
    ensure_request_guard(&options)?;
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
        let resources = options.guards.web_resource_denied.load(Ordering::Relaxed);
        println!(
            "LINTRA_RUNTIME_SECURITY_ACK {{\"navigation_denied\":{navigation},\"new_window_denied\":{popups},\"web_resource_denied\":{resources}}}"
        );
    } else if report.security.is_some() {
        return Err("security report outside qualification mode");
    }
    if options.profile_test {
        let profile = report.profile.as_ref().ok_or("profile report required")?;
        let physical = window.inner_size().map_err(|_| "cannot read native size")?;
        let scale = (window
            .scale_factor()
            .map_err(|_| "cannot read native scale")?
            * 1000.0)
            .round() as u32;
        println!(
            "LINTRA_RUNTIME_PROFILE_NATIVE {}",
            serde_json::json!({
                "scale_milli": scale,
                "physical_width": physical.width,
                "physical_height": physical.height,
                "webview_version": tauri::webview_version().map_err(|_| "cannot read WebView version")?,
                "app_local_data_root": app.path().app_local_data_dir().map_err(|_| "cannot resolve app data")?,
            })
        );
        if !profile.matches_native(&report, (physical.width, physical.height), scale) {
            app.exit(1);
            return Err("profile measurements failed");
        }
    } else if report.profile.is_some() {
        return Err("profile report outside qualification mode");
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
    let profile_test = std::env::args().skip(1).any(|arg| arg == "--profile-test");
    assert!(
        !(security_test && profile_test),
        "qualification modes are exclusive"
    );
    let guards = Arc::new(GuardCounters::default());
    tauri::Builder::default()
        .manage(ProbeOptions {
            exit_after_render,
            security_test,
            profile_test,
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
            if profile_test {
                window = window.initialization_script("window.__LINTRA_PROFILE_TEST__ = true;");
            }
            let built_window = window.build()?;
            #[cfg(target_os = "windows")]
            windows_request_guard::install(&built_window, guards.clone())?;
            #[cfg(not(target_os = "windows"))]
            let _ = built_window;
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
    fn profile_rejects_font_fallback_zoom_and_native_size_mismatch() {
        let report = RenderReport {
            protocol: 1,
            width: 900,
            height: 600,
            security: None,
            profile: None,
        };
        let mut profile = ProfileReport {
            scale_milli: 1500,
            body_font_milli: 14000,
            title_font_milli: 28000,
            normal_face_loaded: true,
            semibold_face_loaded: true,
            body_uses_plex: true,
            title_uses_plex: true,
            no_horizontal_overflow: true,
            title_inside_viewport: true,
        };
        assert!(profile.matches_native(&report, (1350, 900), 1500));
        // CSS/browser zoom alone and stale physical geometry cannot qualify.
        assert!(!profile.matches_native(&report, (900, 600), 1000));
        assert!(!profile.matches_native(&report, (1350, 900), 1000));
        assert!(!profile.matches_native(&report, (1350, 910), 1500));
        profile.normal_face_loaded = false;
        assert!(!profile.matches_native(&report, (1350, 900), 1500));
        profile.normal_face_loaded = true;
        profile.title_inside_viewport = false;
        assert!(!profile.matches_native(&report, (1350, 900), 1500));
        profile.title_inside_viewport = true;
        profile.body_font_milli = 16000;
        assert!(!profile.matches_native(&report, (1350, 900), 1500));
        let mut wire = serde_json::to_value(&profile).unwrap();
        wire["arbitrary_text"] = serde_json::json!("rejected");
        assert!(serde_json::from_value::<ProfileReport>(wire).is_err());
    }

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
                    security: None,
                    profile: None,
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

    #[test]
    fn resource_allowlist_adds_only_the_exact_internal_ipc_origins() {
        for url in [
            "http://ipc.localhost/runtime_probe",
            "ipc://localhost/",
            "http://tauri.localhost/assets/index.js",
            "tauri://localhost/index.html",
        ] {
            assert!(local_resource_origin(&Url::parse(url).unwrap()), "{url}");
        }
        for url in [
            "http://127.0.0.1:49183/navigation",
            "http://ipc.localhost.evil/",
            "http://user@ipc.localhost/",
            "http://ipc.localhost:49183/",
            "https://ipc.localhost/",
            "file:///secret",
        ] {
            assert!(!local_resource_origin(&Url::parse(url).unwrap()), "{url}");
        }
    }
}
