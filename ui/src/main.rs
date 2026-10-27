use slint::ComponentHandle;

fn main() -> Result<(), slint::PlatformError> {
    lintra_ui::select_native_software_renderer()?;
    let window = lintra_ui::MainWindow::new()?;
    window.set_version(env!("CARGO_PKG_VERSION").into());
    window.run()
}
