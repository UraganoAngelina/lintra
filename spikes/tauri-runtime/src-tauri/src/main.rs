#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State, WebviewWindow};

struct ProbeOptions {
    exit_after_render: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RenderReport {
    protocol: u8,
    width: u32,
    height: u32,
}

/// Check the actual caller as well as the capability. No path, file content,
/// schema, keys or arbitrary strings cross this probe's IPC boundary.
fn authorize(window: &WebviewWindow) -> Result<(), &'static str> {
    if window.label() != "probe" {
        return Err("unexpected probe window");
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
    if report.protocol != 1
        || !(1..=32768).contains(&report.width)
        || !(1..=32768).contains(&report.height)
    {
        return Err("invalid runtime render acknowledgment");
    }
    println!(
        "LINTRA_RUNTIME_RENDER_ACK {}",
        serde_json::to_string(&report).map_err(|_| "cannot encode runtime acknowledgment")?
    );
    if options.exit_after_render {
        app.exit(0);
    }
    Ok(())
}

fn main() {
    // Smoke mode exits only after a real React render and Rust round trip.
    // Normal mode remains open for native window inspection and capture.
    let exit_after_render = std::env::args().skip(1).any(|arg| arg == "--smoke-test");
    tauri::Builder::default()
        .manage(ProbeOptions { exit_after_render })
        .invoke_handler(tauri::generate_handler![runtime_probe, render_ack])
        .run(tauri::generate_context!())
        .expect("failed to run the isolated Tauri runtime probe");
}
