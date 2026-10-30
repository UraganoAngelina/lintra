#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
use slint::ComponentHandle;

fn main() -> Result<(), slint::PlatformError> {
    // Packaging can inspect version/provenance without opening a desktop or
    // requiring the operator to install Cargo/Git at runtime.
    if std::env::args_os().skip(1).collect::<Vec<_>>() == [std::ffi::OsString::from("--version")] {
        println!(
            "lintra-ui {} (commit {})",
            env!("CARGO_PKG_VERSION"),
            env!("LINTRA_BUILD_COMMIT")
        );
        return Ok(());
    }
    #[cfg(feature = "embedded-bundle")]
    if std::env::args_os().skip(1).collect::<Vec<_>>()
        == [std::ffi::OsString::from("--packaging-check")]
    {
        return lintra_ui::packaging::self_check();
    }
    lintra_ui::select_native_software_renderer()?;
    let window = lintra_ui::MainWindow::new()?;
    window.set_version(env!("CARGO_PKG_VERSION").into());
    let bundle = std::env::current_exe()
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
        .and_then(|exe| {
            lintra_ui::analysis::AnalysisBundle::resolve(
                &lintra::BundleDirectories::default(),
                &lintra::SystemBundleEnvironment,
                &exe,
            )
        });
    let configuration = lintra_ui::controller::install_configuration(&window, bundle);
    let worker = lintra_ui::worker_controller::install_worker(&window, configuration);
    let results = lintra_ui::results::controller::install_results(&window, worker.clone());
    let exports = lintra_ui::export_controller::install_exports(&window, results.clone());
    let startup_probe = std::env::args_os().skip(1).collect::<Vec<_>>()
        == [std::ffi::OsString::from("--packaging-startup-check")];
    if startup_probe {
        // Exercise the normal native window/event loop, then exit predictably.
        slint::Timer::single_shot(std::time::Duration::from_millis(500), || {
            let _ = slint::quit_event_loop();
        });
    }
    let result = window.run();
    exports.borrow_mut().shutdown();
    results.borrow_mut().shutdown();
    worker.borrow_mut().shutdown();
    if startup_probe && result.is_ok() {
        println!("ui_native_startup=ok");
    }
    result
}
