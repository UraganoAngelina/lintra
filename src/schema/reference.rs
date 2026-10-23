//! Membership declarations are compiled, never inferred from file names.
use super::*;
use crate::{DatasetId, DatasetVersion};

impl Checks<'_> {
    pub(super) fn reference_constraint(
        &mut self,
        datasets: &[String],
        metadata: &[RawMetadataBinding],
        trace: Option<&str>,
        field: Option<&str>,
    ) {
        if !metadata.is_empty() {
            self.unsupported(true, trace, field, "reference metadata binding");
        }
        if datasets.is_empty() || datasets.iter().collect::<HashSet<_>>().len() != datasets.len() {
            self.error(
                Code::InvalidSchemaValue,
                trace,
                field,
                "reference union requires nonempty unique declaration IDs",
            );
        }
        for id in datasets {
            if !self
                .raw
                .reference_data
                .iter()
                .any(|dependency| dependency.id == *id)
            {
                self.error(
                    Code::MissingReferenceDatasetDeclaration,
                    trace,
                    field,
                    "reference dataset declaration does not exist",
                );
            }
        }
    }
}

pub(super) fn compile_dependencies(
    raw: &[RawReferenceDataDependency],
) -> Result<Vec<ReferenceDependency>, (Code, String)> {
    let mut ids = HashSet::new();
    let mut keys = HashSet::new();
    raw.iter()
        .map(|dependency| {
            let invalid = || {
                (
                    Code::InvalidSchemaValue,
                    "invalid or duplicate reference dataset declaration".into(),
                )
            };
            if [&dependency.id, &dependency.dataset_id, &dependency.version]
                .iter()
                .any(|value| value.is_empty() || value.trim() != value.as_str())
                || !ids.insert(&dependency.id)
                || !keys.insert((&dependency.dataset_id, &dependency.version))
            {
                return Err(invalid());
            }
            if !dependency.metadata_attributes.is_empty() {
                return Err((
                    Code::UnsupportedConstruct,
                    "reference_data metadata attributes are not supported".into(),
                ));
            }
            // Authoring path syntax is lexical. Filesystem boundary/symlink checks
            // remain mandatory at preparation; compilation never touches the disk.
            if dependency
                .path
                .split('/')
                .any(|part| part.is_empty() || matches!(part, "." | ".."))
                || dependency
                    .path
                    .chars()
                    .any(|c| c.is_control() || matches!(c, '\\' | ':'))
            {
                return Err(invalid());
            }
            if dependency.sha256.len() != 64 {
                return Err(invalid());
            }
            let mut sha256 = [0; 32];
            for (index, pair) in dependency
                .sha256
                .as_bytes()
                .as_chunks::<2>()
                .0
                .iter()
                .enumerate()
            {
                let nibble = |byte| match byte {
                    b'0'..=b'9' => Some(byte - b'0'),
                    b'a'..=b'f' => Some(byte - b'a' + 10),
                    _ => None,
                };
                sha256[index] = nibble(pair[0]).ok_or_else(invalid)? * 16
                    + nibble(pair[1]).ok_or_else(invalid)?;
            }
            Ok(ReferenceDependency {
                key: DatasetKey {
                    id: DatasetId(dependency.dataset_id.clone()),
                    version: DatasetVersion(dependency.version.clone()),
                },
                sha256,
                bundle_path: dependency.path.as_str().into(),
                metadata_attributes: vec![],
            })
        })
        .collect()
}
