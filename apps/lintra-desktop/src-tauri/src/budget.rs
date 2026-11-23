//! D09 soft resource budget for the desktop run (T-MIG-P5-05).
//!
//! The configuration is the versioned JSON of the CLI (`docs/RESOURCE_BUDGETS.md`),
//! read from `resource-budget.json` in the per-user application configuration
//! directory that Tauri resolves for this app. No threshold is invented: an
//! absent file means "not configured" (no soft check, as the CLI without
//! `--resource-budget`), and a present but invalid file blocks the start with
//! an explicit state instead of being skipped.
//!
//! The preflight, the warning and the decision stay in Rust: the core
//! `PreparedResourcePreflight` lives on the worker thread, the renderer only
//! sees aggregate counters and a one-time opaque confirmation handle.
use lintra::{ResourceBudget, ResourceBudgetAudit};
use serde_json::{Value, json};
use std::{io::Read, path::Path};

/// File name inside the application configuration directory.
pub(crate) const FILE_NAME: &str = "resource-budget.json";
/// Hard bound of the configuration read (a few hundred bytes are expected).
/// It protects the IPC thread from a huge or special file; it is not a budget.
const MAX_FILE_BYTES: u64 = 64 * 1024;

/// What the configuration directory contains when a run is requested.
#[derive(Debug, Clone)]
pub(crate) enum BudgetConfig {
    Unconfigured,
    Invalid,
    Configured(ResourceBudget),
}
impl BudgetConfig {
    /// Read the configuration now (each start reads it again, so an edit by
    /// the operator applies to the next run without restarting the app).
    /// `None` directory (not resolvable) cannot hold a file: not configured.
    pub(crate) fn load(directory: Option<&Path>) -> Self {
        let Some(directory) = directory else {
            return Self::Unconfigured;
        };
        match read_bounded(&directory.join(FILE_NAME)) {
            Ok(None) => Self::Unconfigured,
            Ok(Some(bytes)) => match ResourceBudget::parse(&bytes) {
                Ok(budget) => Self::Configured(budget),
                Err(_) => Self::Invalid,
            },
            Err(()) => Self::Invalid,
        }
    }
    /// `resource_budget_state` DTO: state and thresholds, never the path or
    /// the measurement-source text.
    pub(crate) fn dto(&self) -> Value {
        let (state, input, reference) = match self {
            Self::Unconfigured => ("unconfigured", None, None),
            Self::Invalid => ("invalid", None, None),
            Self::Configured(budget) => (
                "configured",
                budget.input_bytes.map(|n| n.to_string()),
                budget.reference_bytes.map(|n| n.to_string()),
            ),
        };
        json!({"schemaVersion":1,"state":state,"input_bytes":input,"reference_bytes":reference})
    }
}

/// `Ok(None)` only when the file does not exist. Anything else that prevents
/// reading a regular file within the bound is an invalid configuration.
fn read_bounded(path: &Path) -> Result<Option<Vec<u8>>, ()> {
    let metadata = match std::fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(()),
    };
    if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES {
        return Err(());
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .and_then(|file| file.take(MAX_FILE_BYTES + 1).read_to_end(&mut bytes))
        .map_err(|_| ())?;
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return Err(());
    }
    Ok(Some(bytes))
}

/// Soft-budget gate of one run, published by the worker for `validation_budget`.
#[derive(Debug, Clone)]
pub(crate) enum Gate {
    /// No configuration: the run starts without a soft check.
    Unconfigured,
    /// The preflight is reading the selected inputs and references.
    Checking,
    /// Within every configured threshold; the run proceeds.
    WithinBudget(ResourceBudgetAudit),
    /// Over a threshold: the worker waits for Annulla / Procedi comunque.
    ConfirmationRequired(ResourceBudgetAudit),
    /// The operator chose Procedi comunque for this exact preflight.
    Proceeded(ResourceBudgetAudit),
    /// Annulla, or cancellation while the warning was shown.
    Declined(ResourceBudgetAudit),
    /// The preflight did not finish (cancel or hard failure); the run
    /// outcome is authoritative.
    NotCompleted,
}
impl Gate {
    pub(crate) fn dto(
        &self,
        run: Value,
        budget: Option<&ResourceBudget>,
        handle: Option<&str>,
    ) -> Value {
        let (state, audit) = match self {
            Self::Unconfigured => ("unconfigured", None),
            Self::Checking => ("checking", None),
            Self::WithinBudget(audit) => ("within_budget", Some(audit)),
            Self::ConfirmationRequired(audit) => ("confirmation_required", Some(audit)),
            Self::Proceeded(audit) => ("proceeded", Some(audit)),
            Self::Declined(audit) => ("declined", Some(audit)),
            Self::NotCompleted => ("not_completed", None),
        };
        let audit = audit.map(|audit| {
            json!({"schemaVersion":1,"input_bytes":audit.input_bytes.to_string(),
                "reference_bytes":audit.reference_bytes.to_string(),
                "missing_reference_files":audit.missing_reference_files.to_string(),
                "input_budget":budget.and_then(|b| b.input_bytes).map(|n| n.to_string()),
                "reference_budget":budget.and_then(|b| b.reference_bytes).map(|n| n.to_string()),
                "input_budget_exceeded":audit.input_budget_exceeded,
                "reference_budget_exceeded":audit.reference_budget_exceeded})
        });
        // The handle is shown only while the decision is pending.
        let handle = matches!(self, Self::ConfirmationRequired(_))
            .then_some(handle)
            .flatten();
        json!({"schemaVersion":1,"run":run,"state":state,"audit":audit,"confirmation_ref":handle})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(dir: &Path, bytes: &[u8]) {
        std::fs::write(dir.join(FILE_NAME), bytes).unwrap();
    }

    #[test]
    fn absent_valid_and_invalid_files_are_three_distinct_states() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(
            BudgetConfig::load(None),
            BudgetConfig::Unconfigured
        ));
        assert!(matches!(
            BudgetConfig::load(Some(dir.path())),
            BudgetConfig::Unconfigured
        ));
        assert_eq!(
            BudgetConfig::load(Some(dir.path())).dto(),
            json!({"schemaVersion":1,"state":"unconfigured","input_bytes":null,"reference_bytes":null})
        );
        write(dir.path(), br#"{"version":1,"measurement_source":"misura","input_bytes":10,"reference_bytes":null}"#);
        assert_eq!(
            BudgetConfig::load(Some(dir.path())).dto(),
            json!({"schemaVersion":1,"state":"configured","input_bytes":"10","reference_bytes":null})
        );
        for invalid in [
            &br#"{"version":1,"measurement_source":"","input_bytes":10,"reference_bytes":null}"#[..],
            br#"{"version":1,"measurement_source":"m","input_bytes":0,"reference_bytes":null}"#,
            br#"{"version":1,"measurement_source":"m","input_bytes":1,"reference_bytes":null,"x":1}"#,
            b"{",
        ] {
            write(dir.path(), invalid);
            assert!(matches!(BudgetConfig::load(Some(dir.path())), BudgetConfig::Invalid));
            assert_eq!(BudgetConfig::load(Some(dir.path())).dto()["state"], "invalid");
        }
        // Oversize and non-regular entries are invalid, never "not configured".
        let mut big = br#"{"version":1,"measurement_source":""#.to_vec();
        big.resize(MAX_FILE_BYTES as usize + 1, b'a');
        write(dir.path(), &big);
        assert!(matches!(
            BudgetConfig::load(Some(dir.path())),
            BudgetConfig::Invalid
        ));
        std::fs::remove_file(dir.path().join(FILE_NAME)).unwrap();
        std::fs::create_dir(dir.path().join(FILE_NAME)).unwrap();
        assert!(matches!(
            BudgetConfig::load(Some(dir.path())),
            BudgetConfig::Invalid
        ));
    }

    #[test]
    fn the_handle_is_published_only_while_a_decision_is_pending() {
        let audit = ResourceBudgetAudit {
            input_bytes: 20,
            reference_bytes: 0,
            missing_reference_files: 0,
            input_budget_exceeded: true,
            reference_budget_exceeded: false,
            proceeded_over_budget: false,
        };
        let budget = ResourceBudget::parse(
            br#"{"version":1,"measurement_source":"m","input_bytes":10,"reference_bytes":null}"#,
        )
        .unwrap();
        let pending = Gate::ConfirmationRequired(audit).dto(json!("r"), Some(&budget), Some("h"));
        assert_eq!(pending["confirmation_ref"], "h");
        assert_eq!(pending["audit"]["input_bytes"], "20");
        assert_eq!(pending["audit"]["input_budget"], "10");
        assert_eq!(pending["audit"]["reference_budget"], Value::Null);
        for gate in [
            Gate::Proceeded(audit),
            Gate::Declined(audit),
            Gate::Checking,
            Gate::NotCompleted,
            Gate::Unconfigured,
        ] {
            assert_eq!(
                gate.dto(json!("r"), Some(&budget), Some("h"))["confirmation_ref"],
                Value::Null
            );
        }
    }
}
