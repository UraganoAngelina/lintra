fn main() {
    // Opt custom commands into the ACL; invoke_handler alone exposes custom
    // commands to all app WebViews by default in Tauri 2.
    tauri_build::try_build(
        tauri_build::Attributes::new().app_manifest(
            tauri_build::AppManifest::new().commands(&["runtime_probe", "render_ack"]),
        ),
    )
    .expect("failed to build the isolated Tauri runtime probe");
}
