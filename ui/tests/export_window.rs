//! Real Slint callbacks, timers, software pixels and real core export writers.
#![cfg(feature = "embedded-bundle")]
use lintra_ui::{
    MainWindow, analysis::AnalysisBundle, controller::install_configuration,
    export_controller::install_exports, results::controller::install_results,
    worker_controller::install_worker,
};
use slint::platform::{
    Platform, WindowAdapter,
    software_renderer::{MinimalSoftwareWindow, RepaintBufferType},
};
use slint::{ComponentHandle, Model, PhysicalSize, Rgb8Pixel, SharedPixelBuffer};
use std::{
    fs,
    path::Path,
    rc::Rc,
    time::{Duration, Instant},
};
struct Headless {
    window: Rc<MinimalSoftwareWindow>,
    started: Instant,
}
impl Platform for Headless {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }
    fn duration_since_start(&self) -> Duration {
        self.started.elapsed()
    }
}
fn wait(mut pending: impl FnMut() -> bool, mut drive: impl FnMut()) {
    let end = Instant::now() + Duration::from_secs(20);
    while pending() {
        assert!(Instant::now() < end, "native operation timed out");
        drive();
        slint::platform::update_timers_and_animations();
        std::thread::sleep(Duration::from_millis(2));
    }
}
fn capture(window: &MinimalSoftwareWindow, dir: &Path, name: &str, scale: u32) {
    let (w, h) = (1280 * scale, 780 * scale);
    let mut pixels = SharedPixelBuffer::<Rgb8Pixel>::new(w, h);
    window.request_redraw();
    assert!(window.draw_if_needed(|renderer| {
        renderer.set_repaint_buffer_type(RepaintBufferType::NewBuffer);
        renderer.render(pixels.make_mut_slice(), w as usize);
    }));
    let mut png = png::Encoder::new(
        fs::File::create(dir.join(format!("{name}-{scale}x.png"))).unwrap(),
        w,
        h,
    );
    png.set_color(png::ColorType::Rgb);
    png.set_depth(png::BitDepth::Eight);
    png.write_header()
        .unwrap()
        .write_image_data(pixels.as_bytes())
        .unwrap();
}
#[test]
fn native_export_is_bound_to_the_run_ignores_filters_and_reports_actual_save_outcomes() {
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Headless {
        window: window.clone(),
        started: Instant::now(),
    }))
    .unwrap();
    let ui = MainWindow::new().unwrap();
    ui.set_version(env!("CARGO_PKG_VERSION").into());
    ui.show().unwrap();
    window.set_size(PhysicalSize::new(1280, 780));
    let config = install_configuration(&ui, AnalysisBundle::builtin());
    let worker = install_worker(&ui, config.clone());
    let results = install_results(&ui, worker.clone());
    let exports = install_exports(&ui, results.clone());
    let input = tempfile::tempdir().unwrap();
    let source = input.path().join("synthetic.txt");
    fs::write(&source, b"001Z  \n").unwrap();
    ui.invoke_choose_flow(0);
    ui.invoke_choose_version(0);
    ui.invoke_choose_trace(0);
    ui.invoke_choose_encoding(0);
    ui.invoke_file_confirmed(source.to_str().unwrap().into());
    ui.invoke_start_analysis();
    wait(|| ui.get_busy(), || ui.invoke_worker_poll());
    assert!(ui.get_results_available());
    ui.invoke_open_results();
    wait(|| ui.get_results_loading(), || {});
    ui.set_filter_field("ABSENT-FIELD".into());
    ui.invoke_apply_results_filter();
    wait(|| ui.get_results_loading(), || {});
    assert_eq!(ui.get_result_rows().row_count(), 0);
    ui.invoke_request_export();
    assert!(ui.get_export_open());
    assert!(ui.get_export_run().contains("controlli locali"));
    assert!(ui.get_export_run().contains("1 diagnostica"));
    assert_eq!(ui.get_export_format(), 0);
    let images = Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/ui-exports");
    fs::create_dir_all(&images).unwrap();
    for scale in [1, 2] {
        window.dispatch_event(slint::platform::WindowEvent::ScaleFactorChanged {
            scale_factor: scale as f32,
        });
        window.set_size(PhysicalSize::new(1280 * scale, 780 * scale));
        for dark in [false, true] {
            ui.set_dark_mode(dark);
            capture(
                &window,
                &images,
                if dark { "export-dark" } else { "export-light" },
                scale,
            );
        }
    }
    ui.invoke_choose_export_format(2);
    let path = input.path().join("saved.csv");
    ui.set_export_path(path.to_str().unwrap().into());
    ui.invoke_confirm_export();
    assert!(ui.get_export_busy());
    // Only real Slint timer delivery drives export completion; no manual poll.
    wait(|| ui.get_export_busy(), || {});
    assert!(!ui.get_export_error());
    assert!(exports.borrow().last_receipt().is_some());
    let csv = fs::read_to_string(&path).unwrap();
    assert_eq!(
        csv.lines().count(),
        2,
        "export includes the diagnostic hidden by the filter"
    );
    assert!(input.path().join("saved.csv.summary.json").exists());
    ui.invoke_confirm_export();
    wait(|| ui.get_export_busy(), || {});
    assert!(ui.get_export_error());
    assert!(ui.get_export_confirmation_required());
    assert!(ui.get_export_note().contains("Conferma"));
    ui.set_export_overwrite(true);
    ui.invoke_confirm_export();
    wait(|| ui.get_export_busy(), || {});
    assert!(!ui.get_export_error());
    ui.invoke_close_export();
    assert!(!ui.get_export_open());
    ui.invoke_toggle_information();
    assert!(ui.get_info_schema().contains("SYNTHETIC / v1"));
    assert!(ui.get_info_bundle().contains("Bundle dell’analisi"));
    for scale in [1, 2] {
        window.dispatch_event(slint::platform::WindowEvent::ScaleFactorChanged {
            scale_factor: scale as f32,
        });
        window.set_size(PhysicalSize::new(1280 * scale, 780 * scale));
        for dark in [false, true] {
            ui.set_dark_mode(dark);
            capture(
                &window,
                &images,
                if dark {
                    "information-dark"
                } else {
                    "information-light"
                },
                scale,
            );
        }
    }
    window.dispatch_event(slint::platform::WindowEvent::ScaleFactorChanged { scale_factor: 1. });
    window.set_size(PhysicalSize::new(640, 480));
    ui.set_show_info(false);
    ui.invoke_request_export();
    window.request_redraw();
    let mut pixels = SharedPixelBuffer::<Rgb8Pixel>::new(640, 480);
    assert!(window.draw_if_needed(|r| {
        r.render(pixels.make_mut_slice(), 640);
    }));
    // A subsequent real validation does not retarget the result currently
    // displayed: export and About use its captured report, not latest worker.
    ui.invoke_close_export();
    let second = input.path().join("second-valid.txt");
    fs::write(&second, b"001A  \n").unwrap();
    ui.invoke_file_confirmed(second.to_str().unwrap().into());
    ui.invoke_start_analysis();
    wait(|| ui.get_busy(), || ui.invoke_worker_poll());
    assert_eq!(
        worker
            .borrow()
            .last_completed()
            .unwrap()
            .outcome
            .summary
            .total_errors,
        0
    );
    ui.set_show_results(true);
    ui.invoke_request_export();
    assert!(ui.get_export_run().contains("1 diagnostica"));
    ui.invoke_choose_export_format(2);
    let preserved = input.path().join("displayed.csv");
    ui.set_export_path(preserved.to_str().unwrap().into());
    ui.invoke_confirm_export();
    wait(|| ui.get_export_busy(), || {});
    assert_eq!(fs::read_to_string(preserved).unwrap().lines().count(), 2);
    exports.borrow_mut().shutdown();
    results.borrow_mut().shutdown();
    worker.borrow_mut().shutdown();
}
