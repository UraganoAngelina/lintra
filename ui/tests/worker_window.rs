//! Exercise actual Slint cross-thread dispatch and responsive UI with real service.
#![cfg(feature = "embedded-bundle")]
use lintra::*;
use lintra_ui::{
    MainWindow, analysis::AnalysisBundle, controller::install_configuration,
    worker_controller::install_worker,
};
use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use slint::platform::{EventLoopProxy, Platform, WindowAdapter};
use slint::{ComponentHandle, PhysicalSize, Rgb8Pixel, SharedPixelBuffer};
use std::{
    fs,
    rc::Rc,
    sync::{
        Arc, Barrier, Mutex,
        mpsc::{self, Receiver, Sender},
    },
    time::Duration,
};

type Event = Box<dyn FnOnce() + Send>;
#[derive(Clone)]
struct Proxy {
    events: Sender<Event>,
    gate: Arc<Mutex<Option<Arc<Barrier>>>>,
}
impl EventLoopProxy for Proxy {
    fn quit_event_loop(&self) -> Result<(), slint::EventLoopError> {
        Ok(())
    }
    fn invoke_from_event_loop(&self, event: Event) -> Result<(), slint::EventLoopError> {
        self.events
            .send(event)
            .map_err(|_| slint::EventLoopError::EventLoopTerminated)?;
        // Fault-control belongs only to this test platform. It pauses delivery
        // FROM the worker after RunStarted, while UI callbacks/render stay live.
        let gate = self.gate.lock().unwrap().take();
        if let Some(gate) = gate {
            gate.wait();
        }
        Ok(())
    }
}
struct Headless {
    window: Rc<MinimalSoftwareWindow>,
    proxy: Proxy,
}
impl Platform for Headless {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }
    fn new_event_loop_proxy(&self) -> Option<Box<dyn EventLoopProxy>> {
        Some(Box::new(self.proxy.clone()))
    }
    fn duration_since_start(&self) -> Duration {
        Duration::ZERO
    }
}
fn finish(ui: &MainWindow, events: &Receiver<Event>) {
    while ui.get_busy() {
        events
            .recv_timeout(Duration::from_secs(15))
            .expect("real Slint wakeup")();
    }
    while let Ok(event) = events.try_recv() {
        event();
    }
}
fn capture(window: &MinimalSoftwareWindow, name: &str) {
    let output =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/ui-worker-visual");
    fs::create_dir_all(&output).unwrap();
    let mut pixels = SharedPixelBuffer::<Rgb8Pixel>::new(1280, 720);
    window.request_redraw();
    assert!(window.draw_if_needed(|renderer| {
        renderer.render(pixels.make_mut_slice(), 1280);
    }));
    let mut encoder = png::Encoder::new(
        fs::File::create(output.join(format!("{name}.png"))).unwrap(),
        1280,
        720,
    );
    encoder.set_color(png::ColorType::Rgb);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(pixels.as_bytes())
        .unwrap();
}
#[test]
fn native_dispatch_shows_completion_violation_error_and_cancellation_as_distinct_states() {
    let (sender, events) = mpsc::channel();
    let gate = Arc::new(Mutex::new(None));
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Headless {
        window: window.clone(),
        proxy: Proxy {
            events: sender,
            gate: gate.clone(),
        },
    }))
    .unwrap();
    let ui = MainWindow::new().unwrap();
    ui.set_version(env!("CARGO_PKG_VERSION").into());
    ui.show().unwrap();
    window.set_size(PhysicalSize::new(1280, 720));
    let configuration = install_configuration(&ui, AnalysisBundle::builtin());
    let worker = install_worker(&ui, configuration);
    let root = tempfile::tempdir().unwrap();
    let previous_directory = std::env::current_dir().unwrap();
    std::env::set_current_dir(root.path()).unwrap();
    let input = root.path().join("input.txt");
    fs::write(&input, b"001A  \n").unwrap();
    ui.invoke_choose_flow(0);
    ui.invoke_choose_version(0);
    ui.invoke_choose_trace(0);
    ui.invoke_choose_encoding(0);
    ui.invoke_file_confirmed("input.txt".into());
    assert!(ui.get_analysis_ready());
    assert!(ui.get_worker_available());
    ui.invoke_start_analysis();
    assert!(ui.get_busy());
    finish(&ui, &events);
    assert_eq!(
        worker.borrow().last_outcome().unwrap().status,
        RunStatus::CompletedValid
    );
    assert!(!ui.get_configuration_error());
    assert!(!ui.get_validation_invalid());
    assert!(!ui.get_execution_partial());
    assert_eq!(ui.get_execution_label(), "Analisi completata");
    fs::write(&input, b"001Z  \n").unwrap();
    ui.invoke_start_analysis();
    finish(&ui, &events);
    assert_eq!(
        worker.borrow().last_outcome().unwrap().status,
        RunStatus::CompletedInvalid
    );
    assert!(!ui.get_configuration_error());
    assert!(ui.get_validation_invalid());
    let old_report = worker
        .borrow()
        .last_completed()
        .unwrap()
        .report
        .as_ref()
        .unwrap()
        .jsonl_path();
    assert!(old_report.exists());
    fs::write(&input, b"\xef\xbb\xbf001A  \n").unwrap();
    ui.invoke_start_analysis();
    finish(&ui, &events);
    assert!(worker.borrow().last_outcome().is_none());
    assert!(matches!(
        worker.borrow().last_error(),
        Some(lintra_ui::worker::WorkerError::Validation(_))
    ));
    assert!(ui.get_configuration_error());
    assert!(!ui.get_validation_invalid());
    assert_eq!(ui.get_execution_label(), "Errore tecnico");
    assert!(!ui.get_preview_note().contains("nessun BOM"));
    capture(&window, "technical-error-light");
    assert!(old_report.exists());
    fs::write(&input, b"001A  \n").unwrap();
    let barrier = Arc::new(Barrier::new(2));
    *gate.lock().unwrap() = Some(barrier.clone());
    ui.invoke_start_analysis();
    let first = events.recv_timeout(Duration::from_secs(15)).unwrap();
    // The service is paused ON its worker, not the UI thread. Render the real
    // window and change theme before requesting cancellation deterministically.
    capture(&window, "running-light");
    ui.set_dark_mode(true);
    capture(&window, "running-dark");
    let mut pixels = SharedPixelBuffer::<Rgb8Pixel>::new(1280, 720);
    window.request_redraw();
    assert!(window.draw_if_needed(|r| {
        r.render(pixels.make_mut_slice(), 1280);
    }));
    assert!(ui.get_busy());
    ui.invoke_cancel_analysis();
    assert_eq!(ui.get_execution_label(), "Annullamento richiesto");
    capture(&window, "cancel-requested-dark");
    barrier.wait();
    first();
    finish(&ui, &events);
    assert_eq!(
        worker.borrow().last_outcome().unwrap().status,
        RunStatus::Cancelled
    );
    assert_eq!(ui.get_execution_label(), "Analisi annullata");
    assert!(ui.get_execution_partial());
    assert!(!ui.get_configuration_error());
    assert!(!ui.get_validation_invalid());
    assert!(old_report.exists());
    capture(&window, "cancelled-dark");
    worker.borrow_mut().shutdown();
    std::env::set_current_dir(previous_directory).unwrap();
}
