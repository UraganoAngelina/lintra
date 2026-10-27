//! Desktop configuration adapter; execution is delegated to ValidationService.
slint::include_modules!();

/// Select the actual native software renderer explicitly. An environment
/// preference cannot silently turn the reference renderer into a GPU backend.
pub fn select_native_software_renderer() -> Result<(), slint::PlatformError> {
    slint::BackendSelector::new()
        .backend_name("winit".into())
        .renderer_name("software".into())
        .select()
}

pub mod analysis;

pub mod controller;

pub mod worker;

pub mod worker_controller;
