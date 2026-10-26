//! Immutable compiled-in bytes with the same binding/format/trust policy as FS.
use super::{
    dataset::{DatasetKey, LookupSet},
    evidence::*,
    filesystem,
    provider::{ReferenceDataError, ReferenceDataProvider, validate_key},
};
use crate::{
    EmbeddedBundle, ReferenceDataPreparer, ReferenceDependency, ReferencePreparationError,
};
use std::{collections::HashMap, io, sync::Arc};

pub struct EmbeddedReferenceDataProvider {
    bundle: EmbeddedBundle,
    dependencies: Vec<ReferenceDependency>,
    indices: HashMap<DatasetKey, usize>,
}
impl EmbeddedReferenceDataProvider {
    pub fn new(
        bundle: EmbeddedBundle,
        dependencies: Vec<ReferenceDependency>,
    ) -> Result<Self, ReferencePreparationError> {
        let indices = filesystem::validate_bindings(&dependencies)
            .map_err(|error| ReferencePreparationError::Provider(Box::new(error)))?;
        Ok(Self {
            bundle,
            dependencies,
            indices,
        })
    }
    pub fn prepare(
        &self,
        policy: DatasetPreparationPolicy,
    ) -> Result<PreparedReferenceEvidence, ReferencePreparationError> {
        self.prepare_selected(&self.dependencies, policy)
    }
    fn prepare_selected(
        &self,
        dependencies: &[ReferenceDependency],
        policy: DatasetPreparationPolicy,
    ) -> Result<PreparedReferenceEvidence, ReferencePreparationError> {
        let prepare = || {
            let mut prepared = PreparedReferenceEvidence {
                sets: HashMap::new(),
                evidence: vec![],
                indices: HashMap::new(),
                policy,
            };
            for dependency in dependencies {
                let mut evidence = filesystem::initial_evidence(dependency, policy);
                let (evidence, set) =
                    match self
                        .bundle
                        .file(&format!("reference-data/{}", dependency.bundle_path))
                    {
                        Some(bytes) => filesystem::decode_bytes(evidence, dependency, bytes)?,
                        None => {
                            evidence.anomalies.push(DatasetAnomaly::Io {
                            operation: DatasetIoOperation::ReadFile,
                            source: Arc::new(io::Error::new(io::ErrorKind::NotFound,
                                "requested reference payload is absent from the embedded bundle")),
                        });
                            (evidence, None)
                        }
                    };
                filesystem::append_prepared(&mut prepared, evidence, set)?;
            }
            Ok::<_, crate::FilesystemReferenceError>(prepared)
        };
        prepare().map_err(|error| ReferencePreparationError::Provider(Box::new(error)))
    }
}
impl ReferenceDataProvider for EmbeddedReferenceDataProvider {
    /// Legacy bool membership remains Strict, never an informational degraded set.
    fn load(&self, key: &DatasetKey) -> Result<Arc<dyn LookupSet>, ReferenceDataError> {
        validate_key(key)?;
        let dependency = self
            .indices
            .get(key)
            .and_then(|index| self.dependencies.get(*index))
            .ok_or_else(|| ReferenceDataError::Unavailable { key: key.clone() })?;
        self.prepare_selected(
            std::slice::from_ref(dependency),
            DatasetPreparationPolicy::Strict,
        )
        .map_err(|source| ReferenceDataError::Provider {
            key: key.clone(),
            source: Box::new(source),
        })?
        .sets
        .get(key)
        .cloned()
        .ok_or_else(|| ReferenceDataError::Unavailable { key: key.clone() })
    }
}

pub struct EmbeddedReferenceDataPreparer {
    bundle: EmbeddedBundle,
}
impl EmbeddedReferenceDataPreparer {
    pub fn new(bundle: EmbeddedBundle) -> Self {
        Self { bundle }
    }
}
impl ReferenceDataPreparer for EmbeddedReferenceDataPreparer {
    fn prepare(
        &self,
        dependencies: &[ReferenceDependency],
        policy: DatasetPreparationPolicy,
    ) -> Result<PreparedReferenceEvidence, ReferencePreparationError> {
        EmbeddedReferenceDataProvider::new(self.bundle.clone(), dependencies.to_vec())?
            .prepare(policy)
    }
}
