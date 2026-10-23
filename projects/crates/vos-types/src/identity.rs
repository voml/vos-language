use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{ArtifactError, AttributeContract, CanonicalType, TypeContractKind};

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
    /// Durable virtual slot of a stored field. It is never inferred from source
    /// order, is never reused once retired, and is absent when durable storage
    /// does not cover the field.
    pub virtual_field_index: Option<u32>,
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
    /// Durable virtual slot of a stored field, absent otherwise.
    pub virtual_field_index: Option<u32>,
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
#[serde(
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
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
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResolvedFieldContract {
    /// Durable field identity.
    pub field_id: u64,
    /// Durable virtual slot of a stored field, absent otherwise.
    pub virtual_field_index: Option<u32>,
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
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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

    /// Validates versions, identities, references, canonical types, and the
    /// content fingerprint of this resolved contract.
    pub fn validate(&self) -> Result<(), ArtifactError> {
        if self.format_version != RESOLVED_CONTRACT_FORMAT_VERSION
            || self.identity_manifest_version != IDENTITY_MANIFEST_VERSION
            || self.schema_fingerprint.len() != 64
            || !self
                .schema_fingerprint
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(ArtifactError {
                code: "RES011".to_owned(),
                message: "invalid resolved contract version or fingerprint".to_owned(),
            });
        }
        let mut type_ids = BTreeSet::new();
        let mut type_by_path = BTreeMap::new();
        let mut path_by_type = BTreeMap::new();
        let mut paths = BTreeSet::new();
        let mut field_ids = BTreeSet::new();
        for item in &self.types {
            if item.type_id == 0
                || !type_ids.insert(item.type_id)
                || item.canonical_path.is_empty()
                || item.canonical_path.iter().any(String::is_empty)
                || !paths.insert(&item.canonical_path)
            {
                return Err(ArtifactError {
                    code: "RES012".to_owned(),
                    message: "invalid resolved type identity".to_owned(),
                });
            }
            type_by_path.insert(item.canonical_path.clone(), item.type_id);
            path_by_type.insert(item.type_id, item.canonical_path.clone());
            let mut names = BTreeSet::new();
            let mut slots = BTreeSet::new();
            for field in &item.fields {
                let slot_conflicts = field
                    .virtual_field_index
                    .is_some_and(|slot| !slots.insert(slot));
                if field.field_id == 0
                    || !field_ids.insert(field.field_id)
                    || field.canonical_name.is_empty()
                    || !names.insert(&field.canonical_name)
                    || slot_conflicts
                {
                    return Err(ArtifactError {
                        code: "RES013".to_owned(),
                        message: "invalid resolved field identity".to_owned(),
                    });
                }
            }
        }
        for item in &self.types {
            for field in &item.fields {
                validate_resolved_type(&field.canonical_type, &type_by_path, &path_by_type)?;
            }
        }
        let expected = resolved_schema_fingerprint(&self.types);
        if expected != self.schema_fingerprint {
            return Err(ArtifactError {
                code: "RES014".to_owned(),
                message: "resolved contract fingerprint does not match content".to_owned(),
            });
        }
        Ok(())
    }
}

fn validate_resolved_type(
    ty: &ResolvedCanonicalType,
    type_by_path: &BTreeMap<Vec<String>, u64>,
    path_by_type: &BTreeMap<u64, Vec<String>>,
) -> Result<(), ArtifactError> {
    match ty {
        ResolvedCanonicalType::User { path, type_id } => {
            if type_by_path.get(path) != Some(type_id) || path_by_type.get(type_id) != Some(path) {
                return Err(ArtifactError {
                    code: "RES015".to_owned(),
                    message: "resolved user type reference is not present in the contract"
                        .to_owned(),
                });
            }
        }
        ResolvedCanonicalType::Reference(inner)
        | ResolvedCanonicalType::Optional(inner)
        | ResolvedCanonicalType::List(inner) => {
            validate_resolved_type(inner, type_by_path, path_by_type)?;
        }
        ResolvedCanonicalType::Generic { path, arguments } => {
            if path.is_empty() || arguments.is_empty() {
                return Err(ArtifactError {
                    code: "RES016".to_owned(),
                    message: "resolved generic type must have a path and arguments".to_owned(),
                });
            }
            for argument in arguments {
                if let ResolvedCanonicalTypeArgument::Type(inner) = argument {
                    validate_resolved_type(inner, type_by_path, path_by_type)?;
                }
            }
        }
        ResolvedCanonicalType::Builtin(path) => {
            if path.len() != 1 || !is_canonical_builtin(path[0].as_str()) {
                return Err(ArtifactError {
                    code: "RES016".to_owned(),
                    message: "resolved builtin type path is invalid".to_owned(),
                });
            }
        }
    }
    Ok(())
}

/// A deterministic identity evolution event between two bound snapshots.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum IdentityChange {
    /// A type was added with a new durable identity.
    TypeAdded {
        /// Durable type identity.
        type_id: u64,
        /// Canonical path of the new type.
        canonical_path: Vec<String>,
    },
    /// A type disappeared and must remain a tombstone in durable history.
    TypeRemoved {
        /// Durable type identity of the removed type.
        type_id: u64,
        /// Last canonical path associated with the identity.
        canonical_path: Vec<String>,
    },
    /// A type kept its identity while its canonical path changed.
    TypeRenamed {
        /// Durable type identity.
        type_id: u64,
        /// Previous canonical path.
        from: Vec<String>,
        /// New canonical path.
        to: Vec<String>,
    },
    /// A type kept its identity while its declaration kind changed.
    TypeKindChanged {
        /// Durable type identity.
        type_id: u64,
        /// Previous declaration kind.
        from: TypeContractKind,
        /// New declaration kind.
        to: TypeContractKind,
    },
    /// A field was added with a new durable identity.
    FieldAdded {
        /// Durable type identity that owns the new field.
        type_id: u64,
        /// Durable field identity.
        field_id: u64,
        /// Canonical field name.
        canonical_name: String,
        /// Durable virtual slot when the field is stored.
        virtual_field_index: Option<u32>,
    },
    /// A field disappeared and must remain a tombstone in durable history.
    FieldRemoved {
        /// Durable type identity that owned the field.
        type_id: u64,
        /// Durable field identity of the removed field.
        field_id: u64,
        /// Last canonical name associated with the identity.
        canonical_name: String,
        /// Last durable virtual slot, absent when storage never covered it.
        virtual_field_index: Option<u32>,
    },
    /// A field kept its identity while its canonical name changed.
    FieldRenamed {
        /// Durable type identity.
        type_id: u64,
        /// Durable field identity.
        field_id: u64,
        /// Previous canonical name.
        from: String,
        /// New canonical name.
        to: String,
    },
    /// A field kept its identity while its durable virtual slot changed, when it
    /// gained one or when it lost one.
    FieldReordered {
        /// Durable type identity.
        type_id: u64,
        /// Durable field identity.
        field_id: u64,
        /// Previous virtual slot, absent when storage did not cover the field.
        from: Option<u32>,
        /// New virtual slot, absent when storage no longer covers the field.
        to: Option<u32>,
    },
    /// A field identity kept its name but changed semantic content.
    FieldChanged {
        /// Durable type identity.
        type_id: u64,
        /// Durable field identity.
        field_id: u64,
    },
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
    /// Last virtual slot associated with the ID, absent for a field that durable
    /// storage never covered.
    pub virtual_field_index: Option<u32>,
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
        let mut active_slots = BTreeSet::new();
        for item in &self.snapshot.types {
            if item.type_id == 0
                || !active_type_ids.insert(item.type_id)
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
                let slot_conflicts = field.virtual_field_index.is_some_and(|slot| {
                    !slots.insert(slot) || !active_slots.insert((item.type_id, slot))
                });
                if field.field_id == 0
                    || !active_field_ids.insert(field.field_id)
                    || field.canonical_name.is_empty()
                    || !names.insert(&field.canonical_name)
                    || slot_conflicts
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
        let mut retired_slots = BTreeSet::new();
        for item in &self.retired_fields {
            let slot_conflicts = item.virtual_field_index.is_some_and(|slot| {
                active_slots.contains(&(item.type_id, slot))
                    || !retired_slots.insert((item.type_id, slot))
            });
            if item.field_id == 0
                || item.type_id == 0
                || !retired_field_ids.insert(item.field_id)
                || active_field_ids.contains(&item.field_id)
                || item.canonical_name.is_empty()
                || item.retired_at_revision > self.revision
                || (!active_type_ids.contains(&item.type_id)
                    && !retired_type_ids.contains(&item.type_id))
                || slot_conflicts
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

/// Computes a stable fingerprint from the fully resolved canonical model.
pub fn resolved_schema_fingerprint(types: &[ResolvedTypeContract]) -> String {
    let mut types = types
        .iter()
        .map(CanonicalResolvedTypeContract::from)
        .collect::<Vec<_>>();
    types.sort_by_key(|item| item.type_id);
    let wire = serde_json::to_vec(&CanonicalResolvedIdentityModel {
        fingerprint_version: IDENTITY_FINGERPRINT_VERSION,
        types,
    })
    .expect("canonical resolved identity model serializes");
    Sha256::digest(wire)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn is_canonical_builtin(name: &str) -> bool {
    matches!(
        name,
        "i8" | "i16"
            | "i32"
            | "i64"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "f32"
            | "f64"
            | "bool"
            | "utf8"
            | "utf16"
            | "uuid"
            | "decimal"
            | "date"
            | "time"
            | "datetime"
            | "bytes"
    )
}

#[derive(Serialize)]
struct CanonicalIdentityModel<'a> {
    fingerprint_version: &'a str,
    types: Vec<CanonicalTypeContract>,
}

#[derive(Serialize)]
struct CanonicalResolvedIdentityModel<'a> {
    fingerprint_version: &'a str,
    types: Vec<CanonicalResolvedTypeContract>,
}

#[derive(Serialize)]
struct CanonicalResolvedTypeContract {
    type_id: u64,
    canonical_path: Vec<String>,
    kind: TypeContractKind,
    fields: Vec<CanonicalResolvedFieldContract>,
}

impl From<&ResolvedTypeContract> for CanonicalResolvedTypeContract {
    fn from(contract: &ResolvedTypeContract) -> Self {
        let mut fields = contract
            .fields
            .iter()
            .map(CanonicalResolvedFieldContract::from)
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

#[derive(Serialize)]
struct CanonicalResolvedFieldContract {
    field_id: u64,
    canonical_name: String,
    canonical_type: ResolvedCanonicalType,
    attributes: Vec<String>,
    default_value: Option<String>,
}

impl From<&ResolvedFieldContract> for CanonicalResolvedFieldContract {
    fn from(field: &ResolvedFieldContract) -> Self {
        let mut attributes = field
            .attributes
            .iter()
            .map(|attribute| attribute.name.clone())
            .collect::<Vec<_>>();
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
        let mut attributes = field
            .attributes
            .iter()
            .map(|attribute| attribute.name.clone())
            .collect::<Vec<_>>();
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

impl BoundFieldContract {
    /// Compares canonical field content, ignoring source spans, attribute order
    /// and the virtual slot, which identity evolution compares separately.
    pub fn canonical_content_eq(&self, other: &Self) -> bool {
        CanonicalFieldContract::from(self) == CanonicalFieldContract::from(other)
    }
}
