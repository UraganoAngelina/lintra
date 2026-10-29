//! Exact reference evidence from a real worker, retained after source changes.
use lintra::*;
use lintra_ui::{analysis::*, worker::*};
use serde_json::{Value, json};
use std::{
    fs,
    sync::{Arc, atomic::AtomicBool},
};
#[derive(Default)]
struct Env;
impl BundleEnvironment for Env {
    fn var_os(&self, _: &str) -> Option<std::ffi::OsString> {
        None
    }
}
#[test]
fn domain_uses_the_verified_run_snapshot_and_rejects_degraded_evidence() {
    use slint::ComponentHandle;
    let window = slint::platform::software_renderer::MinimalSoftwareWindow::new(
        slint::platform::software_renderer::RepaintBufferType::NewBuffer,
    );
    slint::platform::set_platform(Box::new(Headless {
        window,
        started: std::time::Instant::now(),
    }))
    .unwrap();
    let cases: Vec<Value> = serde_json::from_slice(include_bytes!(
        "../../tests/fixtures/reference_membership/union_cases.json"
    ))
    .unwrap();
    let case = &cases[0];
    let bytes: Vec<u8> = case["bytes_hex"]
        .as_str()
        .unwrap()
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect();
    for verified in [true, false] {
        let root = tempfile::tempdir().unwrap();
        let schemas = root.path().join("schemas");
        let refs = root.path().join("references");
        let folder = schemas.join("SYNTHETIC/v1");
        fs::create_dir_all(&folder).unwrap();
        fs::create_dir(&refs).unwrap();
        fs::write(refs.join("A.json"), &bytes).unwrap();
        let mut schema: Value = serde_json::from_slice(include_bytes!(
            "../../bundles/embedded/schemas/SYNTHETIC/v1/flow.json"
        ))
        .unwrap();
        schema["reference_data"] = json!([{"id":"A","dataset_id":"union-A","version":"1","path":"A.json","sha256": if verified {case["sha256"].as_str().unwrap().to_owned()} else {"0".repeat(64)},"metadata_attributes":[]}]);
        schema["traces"][0]["fields"][1]["constraints"] = json!([{"kind":"reference_data","datasets":["A"],"fallback_values":["ZZ"],"metadata_bindings":[]}]);
        fs::write(
            folder.join("flow.json"),
            serde_json::to_vec(&schema).unwrap(),
        )
        .unwrap();
        let input = root.path().join("input.txt");
        fs::write(&input, b"001XX \n").unwrap();
        let bundle = AnalysisBundle::resolve(
            &BundleDirectories {
                schema_dir: Some(schemas),
                reference_data_dir: Some(refs.clone()),
            },
            &Env,
            &root.path().join("lintra-ui"),
        )
        .unwrap();
        let prepared = AnalysisSelection {
            flow: Some("SYNTHETIC".into()),
            version: Some("v1".into()),
            trace: Some("T1".into()),
            encoding: Some(RawResolvedEncoding::Utf8),
            file: Some(input.clone()),
            reports: ReportSelection::default(),
        }
        .prepare(&bundle)
        .unwrap();
        let completed =
            WorkerHandle::start(prepared, CancellationToken::default(), Arc::new(|| {}))
                .unwrap()
                .wait()
                .unwrap();
        let report = completed.report.unwrap();
        let captured = report
            .references()
            .expect("the exact service evidence is captured");
        let key = &captured.datasets()[0].key().clone();
        let native = if verified {
            let ui = lintra_ui::MainWindow::new().unwrap();
            ui.show().unwrap();
            let configuration = lintra_ui::controller::install_configuration(&ui, Ok(bundle));
            let worker = lintra_ui::worker_controller::install_worker(&ui, configuration);
            let results = lintra_ui::results::controller::install_results(&ui, worker.clone());
            ui.invoke_choose_flow(0);
            ui.invoke_choose_version(0);
            ui.invoke_choose_trace(0);
            ui.invoke_choose_encoding(0);
            ui.invoke_file_confirmed(input.to_string_lossy().into_owned().into());
            ui.invoke_start_analysis();
            wait(|| ui.get_busy(), || ui.invoke_worker_poll());
            Some((ui, worker, results))
        } else {
            None
        };
        fs::write(refs.join("A.json"), b"changed after validation").unwrap();
        if let Some((ui, worker, results)) = native {
            ui.invoke_open_results();
            wait(|| ui.get_results_loading(), || {});
            ui.invoke_select_result(0);
            assert!(ui.get_domain_text().contains("ZZ"));
            assert!(ui.get_domain_more());
            ui.invoke_next_domain_page();
            wait(|| !ui.get_domain_text().starts_with("Dataset "), || {});
            assert!(ui.get_domain_text().contains("AA"));
            assert!(
                ui.get_domain_text()
                    .contains(case["sha256"].as_str().unwrap())
            );
            assert!(!ui.get_domain_more());
            ui.invoke_restart_domain();
            assert!(ui.get_domain_text().contains("ZZ"));
            results.borrow_mut().shutdown();
            worker.borrow_mut().shutdown();
            ui.hide().unwrap();
        }
        let page = captured.domain_page(key, None, &AtomicBool::new(false));
        if verified {
            assert_eq!(page.unwrap().values, vec![b"AA".to_vec()]);
        } else {
            assert!(
                page.is_none(),
                "degraded bytes must not become a valid domain"
            );
        }
        assert_eq!(
            captured.datasets()[0].actual_sha256(),
            completed
                .outcome
                .summary
                .reference_validation
                .as_ref()
                .unwrap()
                .datasets[0]
                .actual_sha256
                .as_ref()
        );
    }
}

struct Headless {
    window: std::rc::Rc<slint::platform::software_renderer::MinimalSoftwareWindow>,
    started: std::time::Instant,
}
impl slint::platform::Platform for Headless {
    fn create_window_adapter(
        &self,
    ) -> Result<std::rc::Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }
    fn duration_since_start(&self) -> std::time::Duration {
        self.started.elapsed()
    }
}
fn wait(mut pending: impl FnMut() -> bool, mut drive: impl FnMut()) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while pending() {
        assert!(
            std::time::Instant::now() < deadline,
            "native domain operation timed out"
        );
        drive();
        slint::platform::update_timers_and_animations();
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
}
