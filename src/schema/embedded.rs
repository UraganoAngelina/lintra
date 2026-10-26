//! Embedded acquisition only; registry/compiler retain all executable checks.
use super::repository::{parse_raw_schema, portable_component};
use crate::{
    EmbeddedBundle, InMemorySchemaRepository, RawFlowSchema, SchemaCatalog, SchemaCatalogEntry,
    SchemaCatalogError, SchemaKey, SchemaLoadError, SchemaRepository,
};

pub struct EmbeddedSchemaRepository {
    schemas: InMemorySchemaRepository,
}
impl EmbeddedSchemaRepository {
    pub fn new(bundle: EmbeddedBundle) -> Result<Self, SchemaLoadError> {
        let mut schemas = InMemorySchemaRepository::new();
        for path in bundle
            .manifest()
            .files()
            .keys()
            .filter(|path| path.starts_with("schemas/"))
        {
            let parts = path.split('/').collect::<Vec<_>>();
            if parts.len() != 4
                || parts[3] != "flow.json"
                || !portable_component(parts[1])
                || !portable_component(parts[2])
            {
                return Err(SchemaLoadError::InvalidKey {
                    component: crate::SchemaKeyComponent::FlowId,
                    reason: crate::SchemaKeyError::NonPortablePathComponent,
                });
            }
            let bytes = bundle.file(path).ok_or_else(|| SchemaLoadError::NotFound {
                key: SchemaKey {
                    flow_id: crate::FlowId(parts[1].into()),
                    version: crate::SchemaVersion(parts[2].into()),
                },
            })?;
            let raw = parse_raw_schema(bytes, std::path::Path::new(path))?;
            if raw.schema_format_version != bundle.manifest().schema_format_version() {
                return Err(SchemaLoadError::Bundle(
                    crate::BundleError::InvalidManifest("schema format differs from bundle"),
                ));
            }
            if raw.flow_id != parts[1] || raw.schema_version != parts[2] {
                return Err(SchemaLoadError::KeyMismatch {
                    requested: SchemaKey {
                        flow_id: crate::FlowId(parts[1].into()),
                        version: crate::SchemaVersion(parts[2].into()),
                    },
                    found: SchemaKey {
                        flow_id: crate::FlowId(raw.flow_id),
                        version: crate::SchemaVersion(raw.schema_version),
                    },
                });
            }
            schemas.insert(raw)?;
        }
        Ok(Self { schemas })
    }
}
impl SchemaRepository for EmbeddedSchemaRepository {
    fn load_raw(&self, key: &SchemaKey) -> Result<RawFlowSchema, SchemaLoadError> {
        self.schemas.load_raw(key)
    }
}
impl SchemaCatalog for EmbeddedSchemaRepository {
    fn list_schemas(&self) -> Result<Vec<SchemaCatalogEntry>, SchemaCatalogError> {
        self.schemas.list_schemas()
    }
}
