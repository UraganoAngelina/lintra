use slint::ComponentHandle;

fn main() -> Result<(), slint::PlatformError> {
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
    let _configuration = lintra_ui::controller::install_configuration(&window, bundle);
    window.run()
}
