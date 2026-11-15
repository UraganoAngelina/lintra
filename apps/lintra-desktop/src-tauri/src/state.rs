use crate::{
    configuration::{Configuration, ConfigurationRequest},
    ipc::{self, ErrorCode},
    security::local_asset_origin,
};
use serde_json::json;
use std::sync::{
    Mutex, TryLockError,
    atomic::{AtomicBool, Ordering},
};
use tauri::Url;

#[cfg(target_os = "windows")]
#[derive(Default)]
pub(crate) struct GuardCounters {
    pub resource_guard_ready: AtomicBool,
    pub resource_guard_failed: AtomicBool,
    pub web_resource_denied: std::sync::atomic::AtomicU32,
}

pub(crate) struct AppState {
    gate: Mutex<Option<Configuration>>,
    closed: AtomicBool,
    info_sent: AtomicBool,
    ready: AtomicBool,
    pub smoke: bool,
    #[cfg(target_os = "windows")]
    pub guards: std::sync::Arc<GuardCounters>,
}

impl AppState {
    pub(crate) fn new(smoke: bool) -> Self {
        Self {
            gate: Mutex::new(None),
            closed: AtomicBool::new(false),
            info_sent: AtomicBool::new(false),
            ready: AtomicBool::new(false),
            smoke,
            #[cfg(target_os = "windows")]
            guards: std::sync::Arc::default(),
        }
    }

    pub(crate) fn close(&self) {
        self.closed.store(true, Ordering::Release);
    }
    pub(crate) fn closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }
    pub(crate) fn ready(&self) -> bool {
        self.ready.load(Ordering::Acquire)
    }

    pub(crate) fn handle(
        &self,
        label: &str,
        origin: &Url,
        raw: Option<&[u8]>,
        choose: &dyn Fn() -> Result<Option<std::path::PathBuf>, ErrorCode>,
    ) -> Vec<u8> {
        if label != "main" || !local_asset_origin(origin) {
            return ipc::error(None, ErrorCode::Unauthorized);
        }
        if self.closed.load(Ordering::Acquire) {
            return ipc::error(None, ErrorCode::StaleContext);
        }
        #[cfg(target_os = "windows")]
        if !self.guards.resource_guard_ready.load(Ordering::Acquire)
            || self.guards.resource_guard_failed.load(Ordering::Acquire)
        {
            return ipc::error(None, ErrorCode::TechnicalFailure);
        }
        // One parser/response at a time; framework buffer allocation precedes this boundary.
        let mut permit = match self.gate.try_lock() {
            Ok(permit) => permit,
            Err(TryLockError::WouldBlock) => return ipc::error(None, ErrorCode::Busy),
            Err(TryLockError::Poisoned(_)) => return ipc::error(None, ErrorCode::TechnicalFailure),
        };
        let request = match raw.ok_or(ErrorCode::InvalidMessage).and_then(ipc::decode) {
            Ok(request) => request,
            Err(code) => return ipc::error(None, code),
        };
        let id = &request.request_id;
        if matches!(
            request.operation.as_str(),
            "catalog_page"
                | "configuration_select"
                | "configuration_page"
                | "plan_create"
                | "plan_input_choose"
        ) {
            let command = match ConfigurationRequest::decode(&request.operation, &request.payload) {
                Ok(command) => command,
                Err(code) => return ipc::error(Some(id), code),
            };
            if !self.info_sent.load(Ordering::Acquire) {
                return ipc::error(Some(id), ErrorCode::InvalidState);
            }
            if permit.is_none() {
                match Configuration::resolve() {
                    Ok(configuration) => *permit = Some(configuration),
                    Err(code) => return ipc::error(Some(id), code),
                }
            }
            return permit
                .as_mut()
                .expect("configuration acquired")
                .dispatch(command, id, choose)
                .unwrap_or_else(|code| ipc::error(Some(id), code));
        }
        if !matches!(request.operation.as_str(), "app_info" | "client_ready") {
            return ipc::error(Some(id), ErrorCode::UnknownOperation);
        }
        if request.payload.len() != 1 {
            return ipc::error(Some(id), ErrorCode::InvalidMessage);
        }
        let payload = match request.operation.as_str() {
            "app_info" => {
                json!({"schemaVersion":1,"app_version":env!("CARGO_PKG_VERSION"),"validation_available":false})
            }
            "client_ready" if self.info_sent.load(Ordering::Acquire) => {
                json!({"schemaVersion":1,"ready":true})
            }
            _ => return ipc::error(Some(id), ErrorCode::InvalidState),
        };
        let bytes = match ipc::response(id, payload) {
            Ok(bytes) => bytes,
            Err(code) => return ipc::error(Some(id), code),
        };
        if request.operation == "app_info" {
            self.info_sent.store(true, Ordering::Release);
        } else {
            self.ready.store(true, Ordering::Release);
        }
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn state() -> AppState {
        let state = AppState::new(false);
        #[cfg(target_os = "windows")]
        state
            .guards
            .resource_guard_ready
            .store(true, Ordering::Release);
        state
    }
    fn request(operation: &str, payload: Value) -> Vec<u8> {
        serde_json::to_vec(&json!({"schemaVersion":1,"request_id":"test_1","kind":"request","operation":operation,"payload":payload})).unwrap()
    }
    fn call(state: &AppState, raw: Option<&[u8]>) -> Value {
        serde_json::from_slice(&state.handle(
            "main",
            &Url::parse("tauri://localhost/").unwrap(),
            raw,
            &|| Err(ErrorCode::TechnicalFailure),
        ))
        .unwrap()
    }

    #[test]
    fn catalog_boundary_checks_dtos_before_acquisition_and_preserves_native_selection() {
        let state = state();
        assert_eq!(
            call(
                &state,
                Some(&request(
                    "catalog_page",
                    json!({"schemaVersion":1,"offset":"0"})
                ))
            )["error"]["code"],
            "IPC_INVALID_STATE"
        );
        call(
            &state,
            Some(&request("app_info", json!({"schemaVersion":1}))),
        );
        for payload in [
            json!({"schemaVersion":1,"offset":0}),
            json!({"schemaVersion":1,"offset":"0","path":"SYNTHETIC"}),
        ] {
            assert_eq!(
                call(&state, Some(&request("catalog_page", payload)))["error"]["code"],
                "IPC_INVALID_MESSAGE"
            );
        }
        assert!(state.gate.lock().unwrap().is_none());
        *state.gate.lock().unwrap() =
            Some(Configuration::new(lintra::AnalysisBundle::builtin().unwrap()).unwrap());
        let catalog = call(
            &state,
            Some(&request(
                "catalog_page",
                json!({"schemaVersion":1,"offset":"0"}),
            )),
        );
        let selected = call(
            &state,
            Some(&request(
                "configuration_select",
                json!({"schemaVersion":1,
            "schema_ref":catalog["payload"]["entries"][0]["schema_ref"],"encoding":"utf8"}),
            )),
        );
        assert_eq!(selected["payload"]["flow_id"], "SYNTHETIC");
        let page = call(
            &state,
            Some(&request(
                "configuration_page",
                json!({"schemaVersion":1,
            "configuration_ref":selected["payload"]["configuration_ref"],"offset":"0"}),
            )),
        );
        assert_eq!(page["payload"]["descriptors"][0]["trace_id"], "T1");
        assert!(page["payload"].get("path").is_none());
        assert!(page["payload"].get("raw").is_none());
    }

    #[test]
    fn shared_wire_vectors_match_the_real_dispatcher() {
        let vectors: Value =
            serde_json::from_str(include_str!("../../tests/ipc-vectors.json")).unwrap();
        for vector in vectors.as_array().unwrap() {
            let result = call(
                &state(),
                Some(vector["request"].as_str().unwrap().as_bytes()),
            );
            assert_eq!(result, vector["response"], "{}", vector["name"]);
        }
    }

    #[test]
    fn startup_handshake_has_no_validator_and_commits_after_valid_frames() {
        let state = state();
        assert_eq!(
            call(
                &state,
                Some(&request("client_ready", json!({"schemaVersion":1})))
            )["error"]["code"],
            "IPC_INVALID_STATE"
        );
        assert!(!state.ready());
        let info = call(
            &state,
            Some(&request("app_info", json!({"schemaVersion":1}))),
        );
        assert_eq!(info["payload"]["validation_available"], false);
        assert_eq!(info["request_id"], "test_1");
        assert_eq!(
            call(
                &state,
                Some(&request("client_ready", json!({"schemaVersion":1})))
            )["payload"]["ready"],
            true
        );
        assert!(state.ready());
    }

    #[test]
    fn invalid_calls_and_forged_intentions_never_advance_state() {
        let state = state();
        assert_eq!(call(&state, None)["error"]["code"], "IPC_INVALID_MESSAGE");
        assert_eq!(
            call(&state, Some(b"\xff"))["error"]["code"],
            "IPC_INVALID_MESSAGE"
        );
        for operation in [
            "approve",
            "persist",
            "read_file",
            "execute_sql",
            "validation_start",
        ] {
            assert_eq!(
                call(
                    &state,
                    Some(&request(operation, json!({"schemaVersion":1})))
                )["error"]["code"],
                "IPC_UNKNOWN_OPERATION"
            );
        }
        assert_eq!(
            call(
                &state,
                Some(&request(
                    "app_info",
                    json!({"schemaVersion":1,"path":"SYNTHETIC"})
                ))
            )["error"]["code"],
            "IPC_INVALID_MESSAGE"
        );
        assert!(!state.info_sent.load(Ordering::Acquire));
        assert!(!state.ready());
    }

    #[test]
    fn owner_close_and_bounded_ingress_are_backend_checks() {
        let state = state();
        let raw = request("app_info", json!({"schemaVersion":1}));
        for (label, origin) in [
            ("other", "tauri://localhost/"),
            ("main", "https://tauri.localhost/"),
            ("main", "http://ipc.localhost/"),
        ] {
            let value: Value = serde_json::from_slice(&state.handle(
                label,
                &Url::parse(origin).unwrap(),
                Some(&raw),
                &|| Err(ErrorCode::TechnicalFailure),
            ))
            .unwrap();
            assert_eq!(value["error"]["code"], "IPC_UNAUTHORIZED");
        }
        {
            let _held = state.gate.lock().unwrap();
            assert_eq!(call(&state, Some(&raw))["error"]["code"], "IPC_BUSY");
        }
        assert!(!state.info_sent.load(Ordering::Acquire));
        state.close();
        assert_eq!(
            call(&state, Some(&raw))["error"]["code"],
            "IPC_STALE_CONTEXT"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn webview_guard_readiness_cannot_be_forged_by_the_payload() {
        let state = AppState::new(false);
        let raw = request("app_info", json!({"schemaVersion":1}));
        assert_eq!(
            call(&state, Some(&raw))["error"]["code"],
            "IPC_TECHNICAL_FAILURE"
        );
        state
            .guards
            .resource_guard_ready
            .store(true, Ordering::Release);
        state
            .guards
            .resource_guard_failed
            .store(true, Ordering::Release);
        assert_eq!(
            call(&state, Some(&raw))["error"]["code"],
            "IPC_TECHNICAL_FAILURE"
        );
    }
}
