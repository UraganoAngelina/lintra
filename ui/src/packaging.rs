//! Read-only packaging probe for the shipped executable, independent of display
//! servers, checkout-relative resources, Cargo or helper executables.
use crate::MainWindow;
use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use slint::platform::{Platform, WindowAdapter};
use slint::{ComponentHandle, PhysicalSize, Rgb8Pixel, SharedPixelBuffer};
use std::rc::Rc;
struct Headless(Rc<MinimalSoftwareWindow>);
impl Platform for Headless {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, slint::PlatformError> {
        Ok(self.0.clone())
    }
    fn duration_since_start(&self) -> std::time::Duration {
        std::time::Duration::ZERO
    }
}
/// A real software frame and embedded catalog, without validating patient data.
/// This is a resource/renderer probe, not proof of X11/Windows window startup.
pub fn self_check() -> Result<(), slint::PlatformError> {
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Headless(window.clone())))
        .map_err(|_| slint::PlatformError::Other("packaging platform unavailable".into()))?;
    let ui = MainWindow::new()?;
    let bundle = crate::analysis::AnalysisBundle::builtin()
        .map_err(|_| slint::PlatformError::Other("embedded catalog unavailable".into()))?;
    if bundle.entries().is_empty() {
        return Err(slint::PlatformError::Other("embedded catalog empty".into()));
    }
    ui.show()?;
    window.set_size(PhysicalSize::new(640, 480));
    let mut pixels = SharedPixelBuffer::<Rgb8Pixel>::new(640, 480);
    window.request_redraw();
    let rendered = window.draw_if_needed(|renderer| {
        renderer.set_repaint_buffer_type(RepaintBufferType::NewBuffer);
        renderer.render(pixels.make_mut_slice(), 640);
    });
    if !rendered || pixels.as_bytes().iter().all(|value| *value == 0) {
        return Err(slint::PlatformError::Other("software frame missing".into()));
    }
    ui.hide()?;
    println!(
        "ui_self_check=ok schemas={} pixels=307200",
        bundle.entries().len()
    );
    Ok(())
}
