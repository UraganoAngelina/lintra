//! Real service JSONL -> asynchronous offset index -> native virtualized table.
#![cfg(feature = "embedded-bundle")]
use lintra::*;
use lintra_ui::{
    MainWindow,
    analysis::AnalysisBundle,
    controller::install_configuration,
    results::{JsonlModel, ReportIndex, ResultsFilter, controller::install_results},
    worker_controller::install_worker,
};
use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use slint::platform::{Platform, WindowAdapter};
use slint::{ComponentHandle, Model, ModelRc, PhysicalSize, Rgb8Pixel, SharedPixelBuffer};
use std::{
    fs,
    io::{BufWriter, Write},
    path::Path,
    rc::Rc,
    sync::atomic::AtomicBool,
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
fn wait_for(mut pending: impl FnMut() -> bool, mut drive: impl FnMut()) {
    let end = Instant::now() + Duration::from_secs(30);
    while pending() {
        assert!(Instant::now() < end, "asynchronous UI operation timed out");
        drive();
        slint::platform::update_timers_and_animations();
        std::thread::sleep(Duration::from_millis(2));
    }
}
fn render(window: &MinimalSoftwareWindow, width: u32, height: u32) -> Vec<u8> {
    let mut pixels = SharedPixelBuffer::<Rgb8Pixel>::new(width, height);
    window.request_redraw();
    assert!(window.draw_if_needed(|r| {
        r.set_repaint_buffer_type(RepaintBufferType::NewBuffer);
        r.render(pixels.make_mut_slice(), width as usize);
    }));
    pixels.as_bytes().to_vec()
}
fn capture(window: &MinimalSoftwareWindow, output: &Path, name: &str, scale: u32) {
    let (w, h) = (1280 * scale, 780 * scale);
    let bytes = render(window, w, h);
    fs::write(output.join(format!("{name}-{scale}x.rgb")), &bytes).unwrap();
    let mut writer = png::Encoder::new(
        fs::File::create(output.join(format!("{name}-{scale}x.png"))).unwrap(),
        w,
        h,
    );
    writer.set_color(png::ColorType::Rgb);
    writer
        .write_header()
        .unwrap()
        .write_image_data(&bytes)
        .unwrap();
}
#[test]
fn real_results_filter_selection_large_disk_scrolling_and_rendering_are_qualified() {
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Headless {
        window: window.clone(),
        started: Instant::now(),
    }))
    .unwrap();
    let ui = MainWindow::new().unwrap();
    ui.show().unwrap();
    window.set_size(PhysicalSize::new(1280, 780));
    let configuration = install_configuration(&ui, AnalysisBundle::builtin());
    let worker = install_worker(&ui, configuration);
    let results = install_results(&ui, worker.clone());
    let input = tempfile::tempdir().unwrap();
    let path = input.path().join("synthetic.txt");
    fs::write(&path, b"001Z  \n".repeat(1000)).unwrap();
    ui.invoke_choose_flow(0);
    ui.invoke_choose_version(0);
    ui.invoke_choose_trace(0);
    ui.invoke_choose_encoding(0);
    ui.invoke_file_confirmed(path.to_string_lossy().into_owned().into());
    ui.invoke_start_analysis();
    wait_for(|| ui.get_busy(), || ui.invoke_worker_poll());
    assert!(ui.get_results_available());
    assert_eq!(
        worker.borrow().last_outcome().unwrap().status,
        RunStatus::CompletedInvalid
    );
    ui.invoke_open_results();
    assert!(ui.get_show_results());
    wait_for(|| ui.get_results_loading(), || {});
    assert!(!ui.get_results_error());
    assert_eq!(ui.get_result_rows().row_count(), 1000);
    assert!(ui.get_results_scope().contains("Controlli locali"));
    assert!(ui.get_results_summary().starts_with("Controlli locali."));
    ui.invoke_select_result(0);
    assert!(ui.get_domain_text().contains("\"A\""));
    assert!(ui.get_domain_text().contains("\"É\""));
    assert!(!ui.get_domain_more());
    let page = ui.get_domain_text();
    ui.invoke_restart_domain();
    assert_eq!(ui.get_domain_text(), page);
    // Stable product navigation must return to the same immutable report.
    // These are actual pointer events through MainWindow, not setter-only checks.
    for (x, expected_results) in [(190.0, false), (345.0, true)] {
        let position = slint::LogicalPosition::new(x, 32.0);
        for event in [
            slint::platform::WindowEvent::PointerMoved { position },
            slint::platform::WindowEvent::PointerPressed {
                position,
                button: slint::platform::PointerEventButton::Left,
            },
            slint::platform::WindowEvent::PointerReleased {
                position,
                button: slint::platform::PointerEventButton::Left,
            },
        ] {
            window.dispatch_event(event);
        }
        assert_eq!(ui.get_show_results(), expected_results);
        assert_eq!(ui.get_selected_result(), 0);
        assert_eq!(ui.get_result_rows().row_count(), 1000);
    }
    let first = ui.get_result_detail();
    assert_eq!(first.line, "1");
    assert!(first.code.starts_with("E_"));
    // Concurrent debug/release qualification must never overwrite one another's
    // captures or timing evidence on a developer machine.
    let directory = if cfg!(debug_assertions) {
        "ui-results-visual"
    } else {
        "ui-results-release-visual"
    };
    let output = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../target")
        .join(directory);
    fs::create_dir_all(&output).unwrap();
    for scale in [1u32, 2] {
        window.dispatch_event(slint::platform::WindowEvent::ScaleFactorChanged {
            scale_factor: scale as f32,
        });
        window.set_size(PhysicalSize::new(1280 * scale, 780 * scale));
        for (dark, name) in [(false, "light"), (true, "dark")] {
            ui.set_dark_mode(dark);
            capture(&window, &output, name, scale);
        }
    }
    window.dispatch_event(slint::platform::WindowEvent::ScaleFactorChanged { scale_factor: 1.0 });
    window.set_size(PhysicalSize::new(1280, 780));
    render(&window, 1280, 780);
    let position = slint::LogicalPosition::new(60.0, 418.0);
    for event in [
        slint::platform::WindowEvent::PointerMoved { position },
        slint::platform::WindowEvent::PointerPressed {
            position,
            button: slint::platform::PointerEventButton::Left,
        },
        slint::platform::WindowEvent::PointerReleased {
            position,
            button: slint::platform::PointerEventButton::Left,
        },
    ] {
        window.dispatch_event(event);
    }
    assert_eq!(
        ui.get_selected_result(),
        1,
        "pointer must select the second row"
    );
    for key in [slint::platform::Key::DownArrow, slint::platform::Key::Home] {
        let text: slint::SharedString = key.into();
        window.dispatch_event(slint::platform::WindowEvent::KeyPressed { text: text.clone() });
        window.dispatch_event(slint::platform::WindowEvent::KeyReleased { text });
        if key == slint::platform::Key::DownArrow {
            assert_eq!(ui.get_selected_result(), 2);
        }
    }
    assert_eq!(ui.get_selected_result(), 0);
    ui.set_results_scroll_y(-17_000.0);
    render(&window, 1280, 780);
    assert_eq!(ui.get_selected_result(), 0);
    assert_eq!(ui.get_result_detail(), first);
    assert!(results.borrow().model().unwrap().decoded_rows() < 200);
    // Filtering preserves a selection by JSONL offset when present, and clears it
    // when absent. Invalid input has an actionable error instead of an empty pass.
    ui.set_filter_line("1".into());
    ui.invoke_apply_results_filter();
    wait_for(|| ui.get_results_loading(), || {});
    assert_eq!(ui.get_result_rows().row_count(), 1);
    assert_eq!(ui.get_selected_result(), 0);
    ui.set_filter_line("2".into());
    ui.invoke_apply_results_filter();
    wait_for(|| ui.get_results_loading(), || {});
    assert_eq!(ui.get_result_rows().row_count(), 1);
    assert_eq!(ui.get_selected_result(), -1);
    ui.set_filter_line("0".into());
    ui.invoke_apply_results_filter();
    assert!(ui.get_results_error());
    ui.set_filter_line("2000".into());
    ui.invoke_apply_results_filter();
    wait_for(|| ui.get_results_loading(), || {});
    assert_eq!(ui.get_result_rows().row_count(), 0);
    assert!(ui.get_results_note().contains("Nessuna diagnostica"));
    // A real worker report supplies the load row and authoritative wire shape.
    let (report, mut summary) = {
        let state = worker.borrow();
        let completed = state.last_completed().unwrap();
        (
            completed.report.as_ref().unwrap().clone(),
            completed.outcome.summary.clone(),
        )
    };
    let bytes = fs::read(report.jsonl_path()).unwrap();
    let first_line = bytes.split_inclusive(|v| *v == b'\n').next().unwrap();
    let mut measurements = Vec::new();
    for count in [300_000usize, 1_000_000] {
        let mut file = tempfile::tempfile().unwrap();
        {
            let mut out = BufWriter::new(&mut file);
            for _ in 0..count {
                out.write_all(first_line).unwrap();
            }
            out.flush().unwrap();
        }
        summary.records_processed = count as u64;
        summary.invalid_records = count as u64;
        summary.total_errors = count as u64;
        for value in summary.errors_by_code.values_mut() {
            *value = count as u64;
        }
        let index = ReportIndex::scan(
            file,
            &summary,
            &ResultsFilter::default(),
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(index.offset_bytes(), count * 8);
        let model = Rc::new(JsonlModel::new(index));
        for scale in [1u32, 2] {
            window.dispatch_event(slint::platform::WindowEvent::ScaleFactorChanged {
                scale_factor: scale as f32,
            });
            let (width, height) = (1280 * scale, 780 * scale);
            window.set_size(PhysicalSize::new(width, height));
            ui.set_result_rows(ModelRc::default());
            ui.set_result_rows(ModelRc::from(model.clone()));
            ui.set_selected_result(-1);
            ui.set_results_scroll_y(0.0);
            render(&window, width, height);
            let before = model.decoded_rows();
            let mut jumps = Vec::new();
            for n in [1usize, 100, 250, 500, 1000, count / 2, count - 20] {
                let started = Instant::now();
                ui.set_results_scroll_y(-(n as f32) * 36.0);
                render(&window, width, height);
                jumps.push(started.elapsed().as_secs_f64() * 1000.0);
            }
            assert!(
                model.decoded_rows() > before,
                "scrolling must fetch new disk rows"
            );
            assert!(
                model.decoded_rows() < 2000,
                "virtualization must not decode the full model"
            );
            assert!(model.cache_len() <= 128);
            ui.set_results_scroll_y(0.0);
            let mut pixels = SharedPixelBuffer::<Rgb8Pixel>::new(width, height);
            window.request_redraw();
            assert!(window.draw_if_needed(|r| {
                r.set_repaint_buffer_type(RepaintBufferType::NewBuffer);
                r.render(pixels.make_mut_slice(), width as usize);
            }));
            let mut frames = Vec::new();
            for _ in 0..30 {
                let start = Instant::now();
                window.dispatch_event(slint::platform::WindowEvent::PointerScrolled {
                    position: slint::LogicalPosition::new(350.0, 450.0),
                    delta_x: 0.0,
                    delta_y: -340.0,
                });
                slint::platform::update_timers_and_animations();
                window.request_redraw();
                assert!(window.draw_if_needed(|r| {
                    r.set_repaint_buffer_type(RepaintBufferType::ReusedBuffer);
                    r.render(pixels.make_mut_slice(), width as usize);
                }));
                frames.push(start.elapsed().as_secs_f64() * 1000.0);
            }
            assert!(
                ui.get_results_scroll_y() < -3400.0,
                "real wheel events must advance the viewport"
            );
            assert!(model.highest_requested_row() > 100);
            frames.sort_by(f64::total_cmp);
            measurements.push(
                serde_json::json!({"rows":count,"scale":scale,"offset_bytes":count*8,
                "decoded_rows":model.decoded_rows(),"cache_rows":model.cache_len(),
                "highest_requested_row":model.highest_requested_row(),"full_redraw_jump_ms":jumps,
                "wheel_frames":30,"wheel_median_ms":frames[15],"wheel_p95_ms":frames[28]}),
            );
        }
    }
    fs::write(
        output.join("scrolling.json"),
        serde_json::to_vec_pretty(&measurements).unwrap(),
    )
    .unwrap();
    // Native minimum desktop size remains navigable through the page scroll view.
    window.dispatch_event(slint::platform::WindowEvent::ScaleFactorChanged { scale_factor: 1.0 });
    window.set_size(PhysicalSize::new(640, 480));
    render(&window, 640, 480);
    results.borrow_mut().shutdown();
    worker.borrow_mut().shutdown();
}
