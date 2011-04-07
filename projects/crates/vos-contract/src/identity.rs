use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    ArtifactError, AttributeContract, CanonicalType, CanonicalTypeArgument, ContractDiagnostic,
    SchemaProjection, TypeContractKind,
};

/// Version of the explicit durable identity manifest.
pub const IDENTITY_MANIFEST_VERSION: &str = "vos-identity-manifest-v0";
/// Version of the canonical semantic identity fingerprint.
pub const IDENTITY_FINGERPRINT_VERSION: &str = "vos-identity-fingerprint-v0";
/// Version of the durable identity history artifact.
pub const IDENTITY_HISTORY_FORMAT_VERSION: &str = "vos-identity-history-v0";
/// Version of the resolved contract artifact.
pub const RESOLVED_CONTRACT_FORMAT_VERSION: &str = "vos-resolved-contract-v1";

/// A reviewable identity assignment for one schema projection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IdentityManifest {
    /// Manifest format version.
    pub format_version: String,
    /// Type identities. Array order is not semantic.
    pub types: Vec<TypeIdentity>,
}

impl IdentityManifest {
    /// Serializes the explicit identity manifest using stable field names.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Reads a strict identity manifest without applying it to a projection.
    pub fn from_json(input: &str) -> Result<Self, ArtifactError> {
        serde_json::from_str(input).map_err(|error| ArtifactError {
            code: "ID011".to_owned(),
            message: error.to_string(),
        })
    }
}

/// Durable identity assigned to a named type.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TypeIdentity {
    /// Canonical namespace-qualified path.
    pub canonical_path: Vec<String>,
    /// Durable type ID. Zero is reserved and invalid.
    pub type_id: u64,
    /// Declaration kind attached to this identity.
    pub kind: TypeContractKind,
    /// Active field identities. Array order is not semantic.
    pub fields: Vec<FieldIdentity>,
}

/// Durable identity assigned to a field.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FieldIdentity {
    /// Current canonical field name.
    pub canonical_name: String,
    /// Durable field ID. Zero is reserved and invalid.
    pub field_id: u64,
    /// Durable virtual slot. It is never inferred from source order.
    pub virtual_field_index: u32,
}

/// A projection with externally assigned durable identities.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IdentityBoundProjection {
    /// Identity manifest version.
    pub manifest_version: String,
    /// Bound types in projection source order.
    pub types: Vec<BoundTypeContract>,
}

/// A projected type with a durable type ID.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BoundTypeContract {
    /// Durable type identity.
    pub type_id: u64,
    /// Canonical path.
    pub canonical_path: Vec<String>,
    /// Type kind.
    pub kind: TypeContractKind,
    /// Bound fields in projection source order.
    pub fields: Vec<BoundFieldContract>,
}

/// A projected field with durable field and virtual-slot identities.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BoundFieldContract {
    /// Durable field identity.
    pub field_id: u64,
    /// Durable virtual slot.
    pub virtual_field_index: u32,
    /// Current canonical name.
    pub canonical_name: String,
    /// Canonical type projection.
    pub canonical_type: CanonicalType,
    /// VOS-owned attributes.
    pub attributes: Vec<AttributeContract>,
    /// Default syntax.
    pub default_value: Option<String>,
}

/// A canonical type after user-defined names are bound to durable identities.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", rename_all_fields = "camelCase")]
pub enum ResolvedCanonicalType {
    /// Builtin scalar or standard VOS type name.
    Builtin(Vec<String>),
    /// User-defined type with its durable identity.
    User {
        /// Canonical type path.
        path: Vec<String>,
        /// Durable type identity.
        type_id: u64,
    },
    /// Reference wrapper.
    Reference(Box<ResolvedCanonicalType>),
    /// Optional wrapper.
    Optional(Box<ResolvedCanonicalType>),
    /// List wrapper.
    List(Box<ResolvedCanonicalType>),
    /// Generic type with resolved type arguments.
    Generic {
        /// Generic path.
        path: Vec<String>,
        /// Generic arguments.
        arguments: Vec<ResolvedCanonicalTypeArgument>,
    },
}

/// A resolved generic type argument.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResolvedCanonicalTypeArgument {
    /// Nested resolved type.
    Type(ResolvedCanonicalType),
    /// Literal generic argument.
    Literal(String),
}

/// A resolved field with a durable field identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedFieldContract {
    /// Durable field identity.
    pub field_id: u64,
    /// Durable virtual slot.
    pub virtual_field_index: u32,
    /// Current canonical name.
    pub canonical_name: String,
    /// Resolved canonical type.
    pub canonical_type: ResolvedCanonicalType,
    /// VOS-owned attributes.
    pub attributes: Vec<AttributeContract>,
    /// Default syntax.
    pub default_value: Option<String>,
}

/// A resolved type with durable type and field identities.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedTypeContract {
    /// Durable type identity.
    pub type_id: u64,
    /// Canonical type path.
    pub canonical_path: Vec<String>,
    /// Type kind.
    pub kind: TypeContractKind,
    /// Resolved fields.
    pub fields: Vec<ResolvedFieldContract>,
}

/// An identity-bound projection with all user-defined field types resolved.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedIdentityProjection {
    /// Identity manifest version.
    pub manifest_version: String,
    /// Resolved types in projection order.
    pub types: Vec<ResolvedTypeContract>,
}

/// Versioned semantic contract consumed by database and ORM adapters.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResolvedContract {
    /// Artifact format version.
    pub format_version: String,
    /// Identity manifest version used to bind this contract.
    pub identity_manifest_version: String,
    /// Canonical semantic fingerprint.
    pub schema_fingerprint: String,
    /// Resolved entities in deterministic source-unit order.
    pub types: Vec<ResolvedTypeContract>,
}

impl ResolvedContract {
    /// Serializes this resolved contract using stable field names.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Reads and validates a strict resolved contract artifact.
    pub fn from_json(input: &str) -> Result<Self, ArtifactError> {
        let contract: Self = serde_json::from_str(input).map_err(|error| ArtifactError {
            code: "RES010".to_owned(),
            message: error.to_string(),
        })?;
        contract.validate()?;
        Ok(contract)
    }

    fn validate(&self) -> Result<(), ArtifactError> {
        if self.format_version != RESOLVED_CONTRACT_FORMAT_VERSION
            || self.identity_manifest_version != IDENTITY_MANIFEST_VERSION
            || self.schema_fingerprint.len() != 64
            || !self.schema_fingerprint.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(ArtifactError {
                code: "RES011".to_owned(),
                message: "invalid resolved contract version or fingerprint".to_owned(),
            });
        }
        let mut type_ids = BTreeSet::new();
        let mut paths = BTreeSet::new();
        let mut field_ids = BTreeSet::new();
        for item in &self.types {
            if item.type_id == 0 || !type_ids.insert(item.type_id)
                || item.canonical_path.is_empty()
                || item.canonical_path.iter().any(String::is_empty)
                || !paths.insert(&item.canonical_path)
            {
                return Err(ArtifactError {
                    code: "RES012".to_owned(),
                    message: "invalid resolved type identity".to_owned(),
                });
            }
            let mut names = BTreeSet::new();
            let mut slots = BTreeSet::new();
            for field in &item.fields {
                if field.field_id == 0 || !field_ids.insert(field.field_id)
                    || field.canonical_name.is_empty()
                    || !names.insert(&field.canonical_name)
                    || !slots.insert(field.virtual_field_index)
                {
                    return Err(ArtifactError {
                        code: "RES013".to_owned(),
                        message: "invalid resolved field identity".to_owned(),
                    });
                }
            }
        }
        Ok(())
    }
}

/// A deterministic identity evolution event between two bound snapshots.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
#[allow(missing_docs)]
pub enum IdentityChange {
    /// A type was added with a new durable identity.
    TypeAdded { type_id: u64, canonical_path: Vec<String> },
    /// A type disappeared and must remain a tombstone in durable history.
    TypeRemoved { type_id: u64, canonical_path: Vec<String> },
    /// A type kept its identity while its canonical path changed.
    TypeRenamed { type_id: u64, from: Vec<String>, to: Vec<String> },
    /// A type kept its identity while its declaration kind changed.
    TypeKindChanged { type_id: u64, from: TypeContractKind, to: TypeContractKind },
    /// A field was added with a new durable identity.
    FieldAdded { type_id: u64, field_id: u64, canonical_name: String, virtual_field_index: u32 },
    /// A field disappeared and must remain a tombstone in durable history.
    FieldRemoved { type_id: u64, field_id: u64, canonical_name: String, virtual_field_index: u32 },
    /// A field kept its identity while its canonical name changed.
    FieldRenamed { type_id: u64, field_id: u64, from: String, to: String },
    /// A field kept its identity while its virtual slot changed.
    FieldReordered { type_id: u64, field_id: u64, from: u32, to: u32 },
    /// A field identity kept its name but changed semantic content.
    FieldChanged { type_id: u64, field_id: u64 },
}

/// The explicit, non-allocating evolution result for two identity snapshots.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentityEvolution {
    /// Identity manifest version of the previous snapshot.
    pub from_manifest_version: String,
    /// Identity manifest version of the current snapshot.
    pub to_manifest_version: String,
    /// Fingerprint of the previous snapshot.
    pub from_fingerprint: String,
    /// Fingerprint of the current snapshot.
    pub to_fingerprint: String,
    /// Deterministically ordered changes.
    pub changes: Vec<IdentityChange>,
}

/// Durable history for one identity-bound schema catalog.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IdentityHistory {
    /// Durable history artifact format version.
    pub format_version: String,
    /// Identity manifest version used by the active snapshot.
    pub manifest_version: String,
    /// Monotonic catalog revision.
    pub revision: u64,
    /// Layout revision for slot-affecting changes.
    pub layout_epoch: u64,
    /// Current identity-bound snapshot.
    pub snapshot: IdentityBoundProjection,
    /// Retired type identities that can never be reused.
    pub retired_types: Vec<RetiredTypeIdentity>,
    /// Retired field identities that can never be reused.
    pub retired_fields: Vec<RetiredFieldIdentity>,
}

/// A durable tombstone for a removed type identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RetiredTypeIdentity {
    /// Retired durable type ID.
    pub type_id: u64,
    /// Last canonical path associated with the ID.
    pub canonical_path: Vec<String>,
    /// Revision at which the identity was retired.
    pub retired_at_revision: u64,
}

/// A durable tombstone for a removed field identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RetiredFieldIdentity {
    /// Retired durable field ID.
    pub field_id: u64,
    /// Type identity that owned the field.
    pub type_id: u64,
    /// Last canonical name associated with the ID.
    pub canonical_name: String,
    /// Last virtual slot associated with the ID.
    pub virtual_field_index: u32,
    /// Revision at which the identity was retired.
    pub retired_at_revision: u64,
}

impl IdentityHistory {
    /// Serializes durable identity history using stable field names.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Reads and validates a strict durable identity history artifact.
    pub fn from_json(input: &str) -> Result<Self, ArtifactError> {
        let history: Self = serde_json::from_str(input).map_err(|error| ArtifactError {
            code: "ID020".to_owned(),
            message: error.to_string(),
        })?;
        history.validate()?;
        Ok(history)
    }

    fn validate(&self) -> Result<(), ArtifactError> {
        if self.format_version != IDENTITY_HISTORY_FORMAT_VERSION
            || self.manifest_version != IDENTITY_MANIFEST_VERSION
            || self.snapshot.manifest_version != self.manifest_version
        {
            return Err(ArtifactError {
                code: "ID021".to_owned(),
                message: "unsupported identity history version".to_owned(),
            });
        }
        let mut active_type_ids = BTreeSet::new();
        let mut active_paths = BTreeSet::new();
        let mut active_field_ids = BTreeSet::new();
        for item in &self.snapshot.types {
            if item.type_id == 0 || !active_type_ids.insert(item.type_id)
                || item.canonical_path.is_empty()
                || item.canonical_path.iter().any(String::is_empty)
                || !active_paths.insert(&item.canonical_path)
            {
                return Err(ArtifactError {
                    code: "ID024".to_owned(),
                    message: "invalid active type identity".to_owned(),
                });
            }
            let mut names = BTreeSet::new();
            let mut slots = BTreeSet::new();
            for field in &item.fields {
                if field.field_id == 0 || !active_field_ids.insert(field.field_id)
                    || field.canonical_name.is_empty() || !names.insert(&field.canonical_name)
                    || !slots.insert(field.virtual_field_index)
                {
                    return Err(ArtifactError {
                        code: "ID025".to_owned(),
                        message: "invalid active field identity".to_owned(),
                    });
                }
            }
        }
        let mut retired_type_ids = BTreeSet::new();
        for item in &self.retired_types {
            if item.type_id == 0
                || !retired_type_ids.insert(item.type_id)
                || active_type_ids.contains(&item.type_id)
                || item.canonical_path.is_empty()
                || item.canonical_path.iter().any(String::is_empty)
                || item.retired_at_revision > self.revision
            {
                return Err(ArtifactError {
                    code: "ID022".to_owned(),
                    message: "invalid or reused retired type identity".to_owned(),
                });
            }
        }
        let mut retired_field_ids = BTreeSet::new();
        for item in &self.retired_fields {
            if item.field_id == 0
                || item.type_id == 0
                || !retired_field_ids.insert(item.field_id)
                || active_field_ids.contains(&item.field_id)
                || item.canonical_name.is_empty()
                || item.retired_at_revision > self.revision
                || (!active_type_ids.contains(&item.type_id) && !retired_type_ids.contains(&item.type_id))
            {
                return Err(ArtifactError {
                    code: "ID023".to_owned(),
                    message: "invalid or reused retired field identity".to_owned(),
                });
            }
        }
        Ok(())
    }
}

/// Binds a reviewable identity manifest to a schema projection.
pub fn bind_identity(
    projection: &SchemaProjection,
    manifest: &IdentityManifest,
) -> Result<IdentityBoundProjection, Vec<ContractDiagnostic>> {
    let mut diagnostics = Vec::new();
    if manifest.format_version != IDENTITY_MANIFEST_VERSION {
        diagnostics.push(diagnostic("ID001", "unsupported identity manifest version"));
    }

    let mut type_index = BTreeMap::new();
    let mut type_ids = BTreeSet::new();
    let mut field_ids = BTreeSet::new();
    for identity in &manifest.types {
        if identity.type_id == 0 || !type_ids.insert(identity.type_id) {
            diagnostics.push(diagnostic("ID002", "type ID must be non-zero and unique"));
        }
        if identity.canonical_path.is_empty()
            || identity.canonical_path.iter().any(String::is_empty)
            || type_index
                .insert(identity.canonical_path.clone(), identity)
                .is_some()
        {
            diagnostics.push(diagnostic(
                "ID003",
                "empty or duplicate type path in identity manifest",
            ));
        }
        validate_field_manifest(identity, &mut diagnostics);
        for field in &identity.fields {
            if !field_ids.insert(field.field_id) {
                diagnostics.push(diagnostic(
                    "ID009",
                    "field ID must be unique throughout the manifest",
                ));
            }
        }
    }

    let projection_paths = projection
        .types
        .iter()
        .map(|item| item.canonical_path.clone())
        .collect::<BTreeSet<_>>();
    let manifest_paths = type_index.keys().cloned().collect::<BTreeSet<_>>();
    for missing in projection_paths.difference(&manifest_paths) {
        diagnostics.push(diagnostic(
            "ID004",
            &format!("missing identity for type {}", missing.join("::")),
        ));
    }
    for unknown in manifest_paths.difference(&projection_paths) {
        diagnostics.push(diagnostic(
            "ID005",
            &format!("identity refers to unknown type {}", unknown.join("::")),
        ));
    }

    let mut bound_types = Vec::new();
    if diagnostics.is_empty() {
        for projected in &projection.types {
            let identity = type_index
                .get(&projected.canonical_path)
                .expect("validated type path");
            let fields = identity
                .fields
                .iter()
                .map(|field| (field.canonical_name.clone(), field))
                .collect::<BTreeMap<_, _>>();
            let projected_names = projected
                .fields
                .iter()
                .map(|field| field.canonical_name.clone())
                .collect::<BTreeSet<_>>();
            let identity_names = fields.keys().cloned().collect::<BTreeSet<_>>();
            for missing in projected_names.difference(&identity_names) {
                diagnostics.push(diagnostic(
                    "ID006",
                    &format!("missing identity for field {}", missing),
                ));
            }
            for unknown in identity_names.difference(&projected_names) {
                diagnostics.push(diagnostic(
                    "ID007",
                    &format!("identity refers to unknown field {}", unknown),
                ));
            }
            if identity.kind != projected.kind {
                diagnostics.push(diagnostic("ID008", "identity kind mismatch"));
            }
            if diagnostics.is_empty() {
                bound_types.push(BoundTypeContract {
                    type_id: identity.type_id,
                    canonical_path: projected.canonical_path.clone(),
                    kind: projected.kind,
                    fields: projected
                        .fields
                        .iter()
                        .map(|field| {
                            let identity = fields
                                .get(&field.canonical_name)
                                .expect("validated field name");
                            BoundFieldContract {
                                field_id: identity.field_id,
                                virtual_field_index: identity.virtual_field_index,
                                canonical_name: field.canonical_name.clone(),
                                canonical_type: field.canonical_type.clone(),
                                attributes: field.attributes.clone(),
                                default_value: field.default_value.clone(),
                            }
                        })
                        .collect(),
                });
            }
        }
    }
    if diagnostics.is_empty() {
        Ok(IdentityBoundProjection {
            manifest_version: manifest.format_version.clone(),
            types: bound_types,
        })
    } else {
        Err(diagnostics)
    }
}

/// Computes a stable fingerprint from the identity-bound semantic model.
pub fn schema_fingerprint(bound: &IdentityBoundProjection) -> String {
    let mut types = bound
        .types
        .iter()
        .map(CanonicalTypeContract::from)
        .collect::<Vec<_>>();
    types.sort_by_key(|item| item.type_id);
    let wire = serde_json::to_vec(&CanonicalIdentityModel {
        fingerprint_version: IDENTITY_FINGERPRINT_VERSION,
        types,
    })
    .expect("canonical identity model serializes");
    Sha256::digest(wire)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Compares two explicitly bound snapshots without assigning or reusing IDs.
pub fn compare_identity(
    previous: &IdentityBoundProjection,
    current: &IdentityBoundProjection,
) -> Result<IdentityEvolution, Vec<ContractDiagnostic>> {
    let mut diagnostics = Vec::new();
    let previous_types = previous.types.iter().map(|item| (item.type_id, item)).collect::<BTreeMap<_, _>>();
    let current_types = current.types.iter().map(|item| (item.type_id, item)).collect::<BTreeMap<_, _>>();

    for previous_type in &previous.types {
        if let Some(current_type) = current.types.iter().find(|item| item.canonical_path == previous_type.canonical_path) {
            if current_type.type_id != previous_type.type_id {
                diagnostics.push(diagnostic("ID012", "type path changed durable ID"));
            }
        }
        for previous_field in &previous_type.fields {
            if let Some(current_type) = current.types.iter().find(|item| item.type_id == previous_type.type_id) {
                if let Some(current_field) = current_type.fields.iter().find(|item| item.canonical_name == previous_field.canonical_name) {
                    if current_field.field_id != previous_field.field_id {
                        diagnostics.push(diagnostic("ID013", "field name changed durable ID"));
                    }
                }
            }
        }
    }
    for current_type in &current.types {
        for current_field in &current_type.fields {
            if let Some(previous_type) = previous.types.iter().find(|item| item.fields.iter().any(|field| field.field_id == current_field.field_id)) {
                if previous_type.type_id != current_type.type_id {
                    diagnostics.push(diagnostic("ID014", "field identity moved between types"));
                }
            }
        }
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let mut changes = Vec::new();
    let type_ids = previous_types.keys().chain(current_types.keys()).copied().collect::<BTreeSet<_>>();
    for type_id in type_ids {
        match (previous_types.get(&type_id), current_types.get(&type_id)) {
            (None, Some(current_type)) => changes.push(IdentityChange::TypeAdded {
                type_id,
                canonical_path: current_type.canonical_path.clone(),
            }),
            (Some(previous_type), None) => changes.push(IdentityChange::TypeRemoved {
                type_id,
                canonical_path: previous_type.canonical_path.clone(),
            }),
            (Some(previous_type), Some(current_type)) => {
                if previous_type.canonical_path != current_type.canonical_path {
                    changes.push(IdentityChange::TypeRenamed {
                        type_id,
                        from: previous_type.canonical_path.clone(),
                        to: current_type.canonical_path.clone(),
                    });
                }
                if previous_type.kind != current_type.kind {
                    changes.push(IdentityChange::TypeKindChanged {
                        type_id,
                        from: previous_type.kind,
                        to: current_type.kind,
                    });
                }
                compare_fields(type_id, &previous_type.fields, &current_type.fields, &mut changes);
            }
            (None, None) => unreachable!(),
        }
    }
    Ok(IdentityEvolution {
        from_manifest_version: previous.manifest_version.clone(),
        to_manifest_version: current.manifest_version.clone(),
        from_fingerprint: schema_fingerprint(previous),
        to_fingerprint: schema_fingerprint(current),
        changes,
    })
}

/// Resolves all user-defined names in an identity-bound projection.
pub fn resolve_identity_types(
    bound: &IdentityBoundProjection,
) -> Result<ResolvedIdentityProjection, Vec<ContractDiagnostic>> {
    resolve_identity_units(&[bound])
}

/// Resolves a deterministic collection of already-bound source units together.
pub fn resolve_identity_units(
    units: &[&IdentityBoundProjection],
) -> Result<ResolvedIdentityProjection, Vec<ContractDiagnostic>> {
    let mut diagnostics = Vec::new();
    let manifest_version = units.first().map(|unit| unit.manifest_version.clone()).unwrap_or_else(|| IDENTITY_MANIFEST_VERSION.to_owned());
    let mut types = Vec::new();
    let mut type_paths = BTreeMap::new();
    let mut seen_type_ids = BTreeMap::new();
    let mut field_ids = BTreeSet::new();
    for unit in units {
        if unit.manifest_version != manifest_version {
            diagnostics.push(diagnostic("RES003", "identity manifest version differs across source units"));
        }
        for item in &unit.types {
            if type_paths.insert(item.canonical_path.clone(), item).is_some() {
                diagnostics.push(diagnostic("RES004", "duplicate type path across source units"));
            }
            if item.type_id == 0 {
                diagnostics.push(diagnostic("RES005", "zero type ID in source unit"));
            } else if seen_type_ids.insert(item.type_id, item.canonical_path.clone()).is_some() {
                diagnostics.push(diagnostic("RES007", "duplicate type ID across source units"));
            }
            for field in &item.fields {
                if !field_ids.insert(field.field_id) {
                    diagnostics.push(diagnostic("RES006", "duplicate field ID across source units"));
                }
            }
            types.push(item);
        }
    }
    let type_ids = type_paths
        .iter()
        .map(|(path, item)| (path.clone(), item.type_id))
        .collect::<BTreeMap<_, _>>();
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let types = types
        .into_iter()
        .map(|item| {
            let fields = item
                .fields
                .iter()
                .map(|field| {
                    let canonical_type = resolve_type(
                        &field.canonical_type,
                        &item.canonical_path[..item.canonical_path.len().saturating_sub(1)],
                        &type_ids,
                        &mut diagnostics,
                    );
                    ResolvedFieldContract {
                        field_id: field.field_id,
                        virtual_field_index: field.virtual_field_index,
                        canonical_name: field.canonical_name.clone(),
                        canonical_type,
                        attributes: field.attributes.clone(),
                        default_value: field.default_value.clone(),
                    }
                })
                .collect();
            ResolvedTypeContract {
                type_id: item.type_id,
                canonical_path: item.canonical_path.clone(),
                kind: item.kind,
                fields,
            }
        })
        .collect();
    if diagnostics.is_empty() {
        Ok(ResolvedIdentityProjection {
            manifest_version,
            types,
        })
    } else {
        Err(diagnostics)
    }
}

/// Builds the first strict resolved contract from one bound projection.
pub fn resolve_contract(
    projection: &SchemaProjection,
    manifest: &IdentityManifest,
) -> Result<ResolvedContract, Vec<ContractDiagnostic>> {
    let bound = bind_identity(projection, manifest)?;
    let resolved = resolve_identity_types(&bound)?;
    Ok(ResolvedContract {
        format_version: RESOLVED_CONTRACT_FORMAT_VERSION.to_owned(),
        identity_manifest_version: resolved.manifest_version,
        schema_fingerprint: schema_fingerprint(&bound),
        types: resolved.types,
    })
}

fn resolve_type(
    ty: &CanonicalType,
    namespace: &[String],
    type_ids: &BTreeMap<Vec<String>, u64>,
    diagnostics: &mut Vec<ContractDiagnostic>,
) -> ResolvedCanonicalType {
    match ty {
        CanonicalType::Named(path) => {
            let mut candidates = Vec::new();
            if let Some(type_id) = type_ids.get(path) {
                candidates.push((path.clone(), *type_id));
            }
            if !path.is_empty() {
                let mut relative = namespace.to_vec();
                relative.extend(path.iter().cloned());
                if let Some(type_id) = type_ids.get(&relative) {
                    if !candidates.iter().any(|(_, candidate_id)| candidate_id == type_id) {
                        candidates.push((relative, *type_id));
                    }
                }
            }
            if candidates.len() > 1 {
                diagnostics.push(diagnostic("RES002", &format!("ambiguous type {}", path.join("::"))));
                return ResolvedCanonicalType::Builtin(path.clone());
            }
            if let Some((canonical_path, type_id)) = candidates.first() {
                ResolvedCanonicalType::User {
                    path: canonical_path.clone(),
                    type_id: *type_id,
                }
            } else if path.len() == 1 && is_builtin(path[0].as_str()) {
                ResolvedCanonicalType::Builtin(path.clone())
            } else {
                diagnostics.push(diagnostic("RES001", &format!("unknown type {}", path.join("::"))));
                ResolvedCanonicalType::Builtin(path.clone())
            }
        }
        CanonicalType::Reference(inner) => ResolvedCanonicalType::Reference(Box::new(resolve_type(inner, namespace, type_ids, diagnostics))),
        CanonicalType::Optional(inner) => ResolvedCanonicalType::Optional(Box::new(resolve_type(inner, namespace, type_ids, diagnostics))),
        CanonicalType::List(inner) => ResolvedCanonicalType::List(Box::new(resolve_type(inner, namespace, type_ids, diagnostics))),
        CanonicalType::Generic { path, arguments } => ResolvedCanonicalType::Generic {
            path: path.clone(),
            arguments: arguments
                .iter()
                .map(|argument| match argument {
                    CanonicalTypeArgument::Type(ty) => ResolvedCanonicalTypeArgument::Type(resolve_type(ty, namespace, type_ids, diagnostics)),
                    CanonicalTypeArgument::Literal(value) => ResolvedCanonicalTypeArgument::Literal(value.clone()),
                })
                .collect(),
        },
    }
}

fn is_builtin(name: &str) -> bool {
    matches!(
        name,
        "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64"
            | "f32" | "f64" | "bool" | "utf8" | "utf16" | "uuid" | "decimal"
            | "d128" | "date" | "time" | "datetime" | "bytes"
    )
}

/// Applies an explicitly bound snapshot to durable identity history.
pub fn evolve_identity(
    previous: &IdentityHistory,
    current: IdentityBoundProjection,
) -> Result<IdentityHistory, Vec<ContractDiagnostic>> {
    let mut diagnostics = Vec::new();
    if previous.manifest_version != current.manifest_version {
        diagnostics.push(diagnostic("ID017", "identity manifest version changed"));
    }
    let evolution = match compare_identity(&previous.snapshot, &current) {
        Ok(evolution) => evolution,
        Err(mut errors) => {
            diagnostics.append(&mut errors);
            return Err(diagnostics);
        }
    };
    let retired_type_ids = previous
        .retired_types
        .iter()
        .map(|item| item.type_id)
        .collect::<BTreeSet<_>>();
    let retired_field_ids = previous
        .retired_fields
        .iter()
        .map(|item| item.field_id)
        .collect::<BTreeSet<_>>();
    if current
        .types
        .iter()
        .any(|item| retired_type_ids.contains(&item.type_id))
    {
        diagnostics.push(diagnostic("ID015", "retired type ID cannot be reused"));
    }
    if current.types.iter().flat_map(|item| item.fields.iter()).any(|item| retired_field_ids.contains(&item.field_id)) {
        diagnostics.push(diagnostic("ID016", "retired field ID cannot be reused"));
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let changed = evolution.from_fingerprint != evolution.to_fingerprint;
    let revision = if changed {
        previous
            .revision
            .checked_add(1)
            .ok_or_else(|| vec![diagnostic("ID018", "identity revision exhausted")])?
    } else {
        previous.revision
    };
    let layout_changed = evolution.changes.iter().any(|change| {
        matches!(
            change,
            IdentityChange::TypeAdded { .. }
                | IdentityChange::TypeRemoved { .. }
                | IdentityChange::FieldAdded { .. }
                | IdentityChange::FieldRemoved { .. }
                | IdentityChange::FieldReordered { .. }
        )
    });
    let layout_epoch = if layout_changed {
        previous
            .layout_epoch
            .checked_add(1)
            .ok_or_else(|| vec![diagnostic("ID019", "identity layout epoch exhausted")])?
    } else {
        previous.layout_epoch
    };
    let mut retired_types = previous.retired_types.clone();
    let mut retired_fields = previous.retired_fields.clone();
    for change in &evolution.changes {
        match change {
            IdentityChange::TypeRemoved {
                type_id,
                canonical_path,
            } => retired_types.push(RetiredTypeIdentity {
                type_id: *type_id,
                canonical_path: canonical_path.clone(),
                retired_at_revision: revision,
            }),
            IdentityChange::FieldRemoved {
                type_id,
                field_id,
                canonical_name,
                virtual_field_index,
            } => retired_fields.push(RetiredFieldIdentity {
                field_id: *field_id,
                type_id: *type_id,
                canonical_name: canonical_name.clone(),
                virtual_field_index: *virtual_field_index,
                retired_at_revision: revision,
            }),
            _ => {}
        }
    }
    retired_types.sort_by_key(|item| item.type_id);
    retired_fields.sort_by_key(|item| item.field_id);
    Ok(IdentityHistory {
        format_version: IDENTITY_HISTORY_FORMAT_VERSION.to_owned(),
        manifest_version: current.manifest_version.clone(),
        revision,
        layout_epoch,
        snapshot: current,
        retired_types,
        retired_fields,
    })
}

fn compare_fields(
    type_id: u64,
    previous: &[BoundFieldContract],
    current: &[BoundFieldContract],
    changes: &mut Vec<IdentityChange>,
) {
    let previous_fields = previous.iter().map(|item| (item.field_id, item)).collect::<BTreeMap<_, _>>();
    let current_fields = current.iter().map(|item| (item.field_id, item)).collect::<BTreeMap<_, _>>();
    let field_ids = previous_fields.keys().chain(current_fields.keys()).copied().collect::<BTreeSet<_>>();
    for field_id in field_ids {
        match (previous_fields.get(&field_id), current_fields.get(&field_id)) {
            (None, Some(field)) => changes.push(IdentityChange::FieldAdded {
                type_id,
                field_id,
                canonical_name: field.canonical_name.clone(),
                virtual_field_index: field.virtual_field_index,
            }),
            (Some(field), None) => changes.push(IdentityChange::FieldRemoved {
                type_id,
                field_id,
                canonical_name: field.canonical_name.clone(),
                virtual_field_index: field.virtual_field_index,
            }),
            (Some(previous), Some(current)) => {
                if previous.canonical_name != current.canonical_name {
                    changes.push(IdentityChange::FieldRenamed {
                        type_id,
                        field_id,
                        from: previous.canonical_name.clone(),
                        to: current.canonical_name.clone(),
                    });
                }
                if previous.virtual_field_index != current.virtual_field_index {
                    changes.push(IdentityChange::FieldReordered {
                        type_id,
                        field_id,
                        from: previous.virtual_field_index,
                        to: current.virtual_field_index,
                    });
                }
                if CanonicalFieldContract::from(*previous) != CanonicalFieldContract::from(*current) {
                    changes.push(IdentityChange::FieldChanged { type_id, field_id });
                }
            }
            (None, None) => unreachable!(),
        }
    }
}

#[derive(Serialize)]
struct CanonicalIdentityModel<'a> {
    fingerprint_version: &'a str,
    types: Vec<CanonicalTypeContract>,
}

#[derive(Serialize)]
struct CanonicalTypeContract {
    type_id: u64,
    canonical_path: Vec<String>,
    kind: TypeContractKind,
    fields: Vec<CanonicalFieldContract>,
}

impl From<&BoundTypeContract> for CanonicalTypeContract {
    fn from(contract: &BoundTypeContract) -> Self {
        let mut fields = contract
            .fields
            .iter()
            .map(CanonicalFieldContract::from)
            .collect::<Vec<_>>();
        fields.sort_by_key(|item| item.field_id);
        Self {
            type_id: contract.type_id,
            canonical_path: contract.canonical_path.clone(),
            kind: contract.kind,
            fields,
        }
    }
}

#[derive(PartialEq, Eq, Serialize)]
struct CanonicalFieldContract {
    field_id: u64,
    canonical_name: String,
    canonical_type: CanonicalType,
    attributes: Vec<String>,
    default_value: Option<String>,
}

impl From<&BoundFieldContract> for CanonicalFieldContract {
    fn from(field: &BoundFieldContract) -> Self {
        let mut attributes = field.attributes.iter().map(|attribute| attribute.name.clone()).collect::<Vec<_>>();
        attributes.sort();
        Self {
            field_id: field.field_id,
            canonical_name: field.canonical_name.clone(),
            canonical_type: field.canonical_type.clone(),
            attributes,
            default_value: field.default_value.clone(),
        }
    }
}

fn validate_field_manifest(identity: &TypeIdentity, diagnostics: &mut Vec<ContractDiagnostic>) {
    let mut names = BTreeSet::new();
    let mut ids = BTreeSet::new();
    let mut slots = BTreeSet::new();
    for field in &identity.fields {
        if field.field_id == 0
            || !ids.insert(field.field_id)
            || !slots.insert(field.virtual_field_index)
        {
            diagnostics.push(diagnostic(
                "ID009",
                "field ID must be non-zero and field ID and virtual slot must be unique",
            ));
        }
        if field.canonical_name.is_empty() || !names.insert(field.canonical_name.clone()) {
            diagnostics.push(diagnostic(
                "ID010",
                "empty or duplicate field name in identity manifest",
            ));
        }
    }
}

fn diagnostic(code: &str, message: &str) -> ContractDiagnostic {
    ContractDiagnostic {
        code: code.to_owned(),
        message: message.to_owned(),
        span: None,
    }
}
