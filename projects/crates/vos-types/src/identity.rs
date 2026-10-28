use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::contract::{
    ContractAttribute, MethodStreamMode, OperationCardinality, OperationContract, OperationEffect,
    SemanticCapability, ServiceContract, ServiceMethodContract,
};
use crate::ids::{canonical_digest, identity_hash_u64};
use crate::{
    ArtifactError, AttributeContract, CanonicalScalar, CanonicalType, ContractArtifact,
    OperationKind, TypeContractKind,
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
    /// Canonical scalar shared with the storage layer.
    Scalar(CanonicalScalar),
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
        ResolvedCanonicalType::Scalar(_) => {}
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

// ---------------------------------------------------------------------------
// Stable identity manifest skeleton (P1 common-contract closure)
// ---------------------------------------------------------------------------

/// Format version of the stable identity manifest.
pub const STABLE_IDENTITY_MANIFEST_VERSION: &str = "vos-stable-identity-v1";

/// Opaque 64-bit stable identity shared by every VOS entity kind.
///
/// Zero is reserved and invalid. A stable identity survives renames, reorders
/// and source-unit moves. Its allocation is authoritative and never derived
/// from source order, name or fingerprint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StableId(pub u64);

impl StableId {
    /// The reserved zero identity that no valid entity may carry.
    pub const RESERVED_ZERO: Self = Self(0);

    /// Builds a stable identity from a raw non-zero value. Callers are
    /// responsible for ensuring the value is globally unique within an
    /// artifact family and never reused after retirement.
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw 64-bit value.
    pub fn as_u64(&self) -> u64 {
        self.0
    }

    /// Returns `false` for the reserved zero identity.
    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }
}

impl core::fmt::Display for StableId {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "#{}", self.0)
    }
}

/// Kind of VOS entity that carries a stable identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EntityKind {
    /// Persistent object schema (table).
    Table,
    /// Inline value or class schema.
    Class,
    /// Schema field owned by a type.
    Field,
    /// Query, UDF, micro or macro operation.
    Operation,
    /// Service declaration.
    Service,
    /// Service method.
    Method,
}

impl EntityKind {
    /// Returns `true` when this kind owns fields (types only).
    pub fn is_type(&self) -> bool {
        matches!(self, Self::Table | Self::Class)
    }

    /// Returns `true` when this kind is a type-owned child (field).
    pub fn is_field(&self) -> bool {
        matches!(self, Self::Field)
    }

    /// Returns `true` when this kind is an operation-like declaration.
    pub fn is_operation(&self) -> bool {
        matches!(self, Self::Operation)
    }

    /// Returns `true` when this kind is a service or service method.
    pub fn is_service(&self) -> bool {
        matches!(self, Self::Service | Self::Method)
    }
}

/// Byte span inside a source unit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceSpan {
    /// Start byte offset (inclusive).
    pub start: usize,
    /// End byte offset (exclusive).
    pub end: usize,
}

impl SourceSpan {
    /// Builds a validated span. Empty spans are allowed.
    pub fn new(start: usize, end: usize) -> Option<Self> {
        if start <= end {
            Some(Self { start, end })
        } else {
            None
        }
    }
}

/// Source provenance of a stable identity entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceProvenance {
    /// Source unit id within the artifact.
    pub source_unit_id: u32,
    /// Declaration span inside the source unit when available.
    pub span: Option<SourceSpan>,
}

impl SourceProvenance {
    /// Provenance with only a source unit id and no span.
    pub fn unit(source_unit_id: u32) -> Self {
        Self {
            source_unit_id,
            span: None,
        }
    }

    /// Provenance with both source unit and span.
    pub fn spanned(source_unit_id: u32, span: SourceSpan) -> Self {
        Self {
            source_unit_id,
            span: Some(span),
        }
    }
}

/// One stable identity entry in a manifest.
///
/// Each VOS entity owns exactly one entry. The `stable_id` never changes
/// across rename, reorder or source-unit moves. The `fingerprint` reflects
/// canonical semantic content and must change whenever the entity body
/// changes semantically.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StableIdentityEntry {
    /// Globally unique durable identifier.
    pub stable_id: StableId,
    /// Entity kind (table / class / field / operation / service / method).
    pub entity_kind: EntityKind,
    /// Current canonical name. Subject to rename tracking.
    pub current_name: String,
    /// Canonical semantic fingerprint (64 lowercase hex SHA-256).
    pub fingerprint: String,
    /// Source provenance (source unit and optional span).
    pub provenance: SourceProvenance,
    /// Previous stable identity when this entry tracks a rename chain.
    pub previous: Option<StableId>,
}

/// Manifest of every stable identity in a VOS artifact.
///
/// The manifest is the single source of truth for durable entity identity.
/// Entries are **not** ordered semantically; any traversal that needs a
/// deterministic order must sort by `stable_id` or another explicit key.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StableIdentityManifest {
    /// Manifest format version.
    pub format_version: String,
    /// Unique artifact identifier (e.g. package name + version).
    pub artifact_id: String,
    /// Stable identity entries. Array order is not semantic.
    pub entries: Vec<StableIdentityEntry>,
}

impl StableIdentityManifest {
    /// Builds an empty manifest with the current format version.
    pub fn new(artifact_id: impl Into<String>) -> Self {
        Self {
            format_version: STABLE_IDENTITY_MANIFEST_VERSION.to_owned(),
            artifact_id: artifact_id.into(),
            entries: Vec::new(),
        }
    }

    /// Serializes the manifest using stable field names.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Reads a strict manifest without applying it to a projection.
    pub fn from_json(input: &str) -> Result<Self, ArtifactError> {
        let manifest: Self = serde_json::from_str(input).map_err(|error| ArtifactError {
            code: "SIM010".to_owned(),
            message: error.to_string(),
        })?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Validates version, unique stable ids, non-empty names and fingerprint
    /// shape. Does **not** apply the manifest to a projection.
    pub fn validate(&self) -> Result<(), ArtifactError> {
        if self.format_version != STABLE_IDENTITY_MANIFEST_VERSION {
            return Err(ArtifactError {
                code: "SIM011".to_owned(),
                message: "unsupported stable identity manifest version".to_owned(),
            });
        }
        if self.artifact_id.trim().is_empty() {
            return Err(ArtifactError {
                code: "SIM012".to_owned(),
                message: "stable identity manifest artifact id must not be empty".to_owned(),
            });
        }
        let mut seen = BTreeSet::new();
        for entry in &self.entries {
            if !entry.stable_id.is_valid() {
                return Err(ArtifactError {
                    code: "SIM013".to_owned(),
                    message: "stable identity must be non-zero".to_owned(),
                });
            }
            if !seen.insert(entry.stable_id) {
                return Err(ArtifactError {
                    code: "SIM014".to_owned(),
                    message: format!("duplicate stable identity {}", entry.stable_id),
                });
            }
            if entry.current_name.trim().is_empty() {
                return Err(ArtifactError {
                    code: "SIM015".to_owned(),
                    message: "stable identity entry name must not be empty".to_owned(),
                });
            }
            if entry.fingerprint.len() != 64
                || !entry
                    .fingerprint
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit())
            {
                return Err(ArtifactError {
                    code: "SIM016".to_owned(),
                    message: "stable identity entry fingerprint must be 64 ascii-hex chars"
                        .to_owned(),
                });
            }
            if let Some(previous) = entry.previous
                && !previous.is_valid()
            {
                return Err(ArtifactError {
                    code: "SIM017".to_owned(),
                    message: "stable identity previous link must be non-zero".to_owned(),
                });
            }
        }
        Ok(())
    }

    /// Builds a stable identity manifest from a resolved contract.
    ///
    /// Every [`ResolvedTypeContract`] and its fields receive one entry. The
    /// existing durable `type_id` and `field_id` are reused as the stable id
    /// for the corresponding entries, so reordering or renaming entities
    /// after this manifest is produced still yields the same stable id.
    pub fn from_resolved_contract(contract: &ResolvedContract) -> Self {
        let mut entries = Vec::new();
        for type_contract in &contract.types {
            let kind = match type_contract.kind {
                TypeContractKind::Table => EntityKind::Table,
                TypeContractKind::Class => EntityKind::Class,
            };
            let name = type_contract
                .canonical_path
                .last()
                .cloned()
                .unwrap_or_default();
            entries.push(StableIdentityEntry {
                stable_id: StableId::new(type_contract.type_id),
                entity_kind: kind,
                current_name: name,
                fingerprint: type_fingerprint(type_contract),
                provenance: SourceProvenance::unit(0),
                previous: None,
            });
            for field in &type_contract.fields {
                entries.push(StableIdentityEntry {
                    stable_id: StableId::new(field.field_id),
                    entity_kind: EntityKind::Field,
                    current_name: field.canonical_name.clone(),
                    fingerprint: field_fingerprint(field),
                    provenance: SourceProvenance::unit(0),
                    previous: None,
                });
            }
        }
        Self {
            format_version: STABLE_IDENTITY_MANIFEST_VERSION.to_owned(),
            artifact_id: contract.format_version.clone(),
            entries,
        }
    }

    /// Builds a stable identity manifest from a contract artifact.
    ///
    /// Every [`OperationContract`], [`ServiceContract`] and
    /// [`ServiceMethodContract`] in the artifact receives one entry. The
    /// existing durable string identity (`operation_id`, `service_id`,
    /// `method_id`) is deterministically hashed into a [`StableId`], so the
    /// stable id survives source reorder as long as the durable string
    /// identity does not change.
    pub fn from_artifact(artifact: &ContractArtifact) -> Self {
        Self::from_payload_id(&artifact.payload, artifact_id_from_artifact(artifact))
    }

    /// Builds a stable identity manifest from the union of a resolved contract
    /// and a contract artifact.
    ///
    /// Types and their fields come from the resolved contract; operations,
    /// services and service methods come from the artifact payload. Entries
    /// are merged by kind so duplicate stable ids cannot appear when the two
    /// inputs happen to carry the same value.
    pub fn from_full_artifact(resolved: &ResolvedContract, artifact: &ContractArtifact) -> Self {
        let mut merged = Self::from_resolved_contract(resolved);
        let artifact_entries = Self::from_artifact(artifact).entries;
        let mut seen = merged
            .entries
            .iter()
            .map(|entry| entry.stable_id)
            .collect::<BTreeSet<_>>();
        for entry in artifact_entries {
            if seen.insert(entry.stable_id) {
                merged.entries.push(entry);
            }
        }
        merged.artifact_id = artifact_id_from_artifact(artifact);
        merged
    }

    /// Builds a manifest directly from the artifact payload with a caller
    /// supplied artifact id. Useful for tests and any consumer that already
    /// has the payload decomposed.
    pub fn from_payload_id(payload: &crate::ContractPayload, artifact_id: String) -> Self {
        let mut entries = Vec::new();
        for operation in &payload.operations {
            let stable_id = operation
                .stable_id_hint
                .map(StableId::new)
                .unwrap_or_else(|| {
                    StableId::new(identity_hash_u64(operation.operation_id.as_str()))
                });
            entries.push(StableIdentityEntry {
                stable_id,
                entity_kind: EntityKind::Operation,
                current_name: operation.name.clone(),
                fingerprint: operation_fingerprint(operation),
                provenance: SourceProvenance::spanned(
                    operation.source_unit_id,
                    SourceSpan::new(operation.span.start, operation.span.end).unwrap_or_else(
                        || SourceSpan::new(operation.span.start, operation.span.start).unwrap(),
                    ),
                ),
                previous: None,
            });
        }
        for service in &payload.services {
            let name = service
                .canonical_path
                .last()
                .cloned()
                .unwrap_or_else(|| service.name.clone());
            let stable_id = service
                .stable_id_hint
                .map(StableId::new)
                .unwrap_or_else(|| StableId::new(identity_hash_u64(service.service_id.as_str())));
            entries.push(StableIdentityEntry {
                stable_id,
                entity_kind: EntityKind::Service,
                current_name: name.clone(),
                fingerprint: service_fingerprint(service),
                provenance: SourceProvenance::spanned(
                    service.source_unit_id,
                    SourceSpan::new(service.span.start, service.span.end).unwrap_or_else(|| {
                        SourceSpan::new(service.span.start, service.span.start).unwrap()
                    }),
                ),
                previous: None,
            });
            for method in &service.methods {
                let stable_id = method
                    .stable_id_hint
                    .map(StableId::new)
                    .unwrap_or_else(|| StableId::new(identity_hash_u64(method.method_id.as_str())));
                entries.push(StableIdentityEntry {
                    stable_id,
                    entity_kind: EntityKind::Method,
                    current_name: method.name.clone(),
                    fingerprint: method_fingerprint(method),
                    provenance: SourceProvenance::unit(service.source_unit_id),
                    previous: None,
                });
            }
        }
        Self {
            format_version: STABLE_IDENTITY_MANIFEST_VERSION.to_owned(),
            artifact_id,
            entries,
        }
    }

    /// Verifies that reordering entities does not change the stable id set.
    ///
    /// Returns a list of human-readable error strings describing every
    /// mismatch. The manifest is considered stable under reorder when the
    /// set of `(stable_id, entity_kind)` pairs is identical and no entry's
    /// canonical name changes its stable id.
    pub fn verify_reorder_stable(
        &self,
        before: &StableIdentityManifest,
    ) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        let before_by_id = before
            .entries
            .iter()
            .map(|entry| (entry.stable_id, entry))
            .collect::<BTreeMap<_, _>>();
        let self_by_id = self
            .entries
            .iter()
            .map(|entry| (entry.stable_id, entry))
            .collect::<BTreeMap<_, _>>();

        // No entry may disappear under a pure reorder.
        for (stable_id, before_entry) in &before_by_id {
            if !self_by_id.contains_key(stable_id) {
                errors.push(format!(
                    "reorder removed stable identity {} ({:?})",
                    stable_id, before_entry.current_name
                ));
            }
        }
        // No entry may appear with a new stable id under a pure reorder.
        for (stable_id, self_entry) in &self_by_id {
            if !before_by_id.contains_key(stable_id) {
                errors.push(format!(
                    "reorder introduced new stable identity {} ({:?})",
                    stable_id, self_entry.current_name
                ));
            }
        }

        // Corresponding entries must keep their kind and canonical name.
        for (stable_id, before_entry) in &before_by_id {
            let Some(self_entry) = self_by_id.get(stable_id) else {
                continue;
            };
            if self_entry.entity_kind != before_entry.entity_kind {
                errors.push(format!(
                    "reorder changed entity kind of {} from {:?} to {:?}",
                    stable_id, before_entry.entity_kind, self_entry.entity_kind
                ));
            }
            if self_entry.current_name != before_entry.current_name {
                errors.push(format!(
                    "reorder changed canonical name of {} from {:?} to {:?}",
                    stable_id, before_entry.current_name, self_entry.current_name
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Verifies that renaming an entity keeps the stable id intact.
    ///
    /// A rename is detected when two entries share the same stable id but have
    /// different canonical names. This is always allowed: the stable id is the
    /// durable handle and the name is display-only. If no rename happened the
    /// check succeeds trivially.
    pub fn verify_rename_tracking(
        &self,
        before: &StableIdentityManifest,
    ) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        let before_by_id = before
            .entries
            .iter()
            .map(|entry| (entry.stable_id, entry))
            .collect::<BTreeMap<_, _>>();

        for self_entry in &self.entries {
            let Some(before_entry) = before_by_id.get(&self_entry.stable_id) else {
                continue;
            };
            // Rename must not change entity kind.
            if self_entry.entity_kind != before_entry.entity_kind {
                errors.push(format!(
                    "rename changed entity kind of {} from {:?} to {:?}",
                    self_entry.stable_id, before_entry.entity_kind, self_entry.entity_kind
                ));
            }
            // If name unchanged, fingerprint should also be unchanged (pure
            // reorder is handled by verify_reorder_stable). If name changed,
            // fingerprint may or may not change — either is legal.
            if self_entry.current_name == before_entry.current_name
                && self_entry.fingerprint != before_entry.fingerprint
            {
                errors.push(format!(
                    "entry {} kept its name but changed fingerprint without a rename",
                    self_entry.stable_id
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// ---------------------------------------------------------------------------
// Canonical fingerprint helpers for stable identity entries.
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CanonicalTypeIdentity<'a> {
    fingerprint_version: &'a str,
    type_id: u64,
    canonical_path: &'a [String],
    kind: TypeContractKind,
    fields: Vec<CanonicalFieldIdentity<'a>>,
}

fn type_fingerprint(type_contract: &ResolvedTypeContract) -> String {
    let mut fields = type_contract
        .fields
        .iter()
        .map(CanonicalFieldIdentity::from)
        .collect::<Vec<_>>();
    fields.sort_by_key(|item| item.field_id);
    canonical_digest(&CanonicalTypeIdentity {
        fingerprint_version: IDENTITY_FINGERPRINT_VERSION,
        type_id: type_contract.type_id,
        canonical_path: &type_contract.canonical_path,
        kind: type_contract.kind,
        fields,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CanonicalFieldIdentity<'a> {
    field_id: u64,
    canonical_name: &'a str,
    canonical_type: &'a ResolvedCanonicalType,
    attributes: Vec<String>,
    default_value: Option<&'a str>,
}

impl<'a> From<&'a ResolvedFieldContract> for CanonicalFieldIdentity<'a> {
    fn from(field: &'a ResolvedFieldContract) -> Self {
        let mut attributes = field
            .attributes
            .iter()
            .map(|attribute| attribute.name.clone())
            .collect::<Vec<_>>();
        attributes.sort();
        Self {
            field_id: field.field_id,
            canonical_name: &field.canonical_name,
            canonical_type: &field.canonical_type,
            attributes,
            default_value: field.default_value.as_deref(),
        }
    }
}

fn field_fingerprint(field: &ResolvedFieldContract) -> String {
    canonical_digest(&CanonicalFieldIdentity::from(field))
}

// ---------------------------------------------------------------------------
// Fingerprint helpers for operation / service / method stable entries
// ---------------------------------------------------------------------------

/// Canonical identity of a single operation contract, excluding provenance.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CanonicalOperationIdentity<'a> {
    fingerprint_version: &'a str,
    operation_id: &'a str,
    kind: OperationKind,
    canonical_path: &'a [String],
    parameters: Vec<CanonicalOperationParameter<'a>>,
    result_type: &'a Option<ResolvedCanonicalType>,
    cardinality: OperationCardinality,
    effect: OperationEffect,
    capabilities: Vec<SemanticCapability>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CanonicalOperationParameter<'a> {
    name: &'a str,
    canonical_type: &'a ResolvedCanonicalType,
}

fn operation_fingerprint(operation: &OperationContract) -> String {
    let mut capabilities = operation.capabilities.clone();
    capabilities.sort_unstable();
    capabilities.dedup();
    canonical_digest(&CanonicalOperationIdentity {
        fingerprint_version: IDENTITY_FINGERPRINT_VERSION,
        operation_id: operation.operation_id.as_str(),
        kind: operation.kind,
        canonical_path: &operation.canonical_path,
        parameters: operation
            .parameters
            .iter()
            .map(|parameter| CanonicalOperationParameter {
                name: &parameter.name,
                canonical_type: &parameter.canonical_type,
            })
            .collect(),
        result_type: &operation.result_type,
        cardinality: operation.cardinality,
        effect: operation.effect,
        capabilities,
    })
}

/// Canonical identity of a single service contract, excluding provenance.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CanonicalServiceIdentity<'a> {
    fingerprint_version: &'a str,
    service_id: &'a str,
    canonical_path: &'a [String],
    attributes: Vec<CanonicalAttributeIdentity<'a>>,
}

fn service_fingerprint(service: &ServiceContract) -> String {
    canonical_digest(&CanonicalServiceIdentity {
        fingerprint_version: IDENTITY_FINGERPRINT_VERSION,
        service_id: service.service_id.as_str(),
        canonical_path: &service.canonical_path,
        attributes: canonical_attributes_identity(&service.attributes),
    })
}

/// Canonical identity of a single service method contract.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CanonicalMethodIdentity<'a> {
    fingerprint_version: &'a str,
    method_id: &'a str,
    name: &'a str,
    request_type: &'a ResolvedCanonicalType,
    response_type: &'a ResolvedCanonicalType,
    stream: MethodStreamMode,
    operation_id: &'a str,
    attributes: Vec<CanonicalAttributeIdentity<'a>>,
}

fn method_fingerprint(method: &ServiceMethodContract) -> String {
    canonical_digest(&CanonicalMethodIdentity {
        fingerprint_version: IDENTITY_FINGERPRINT_VERSION,
        method_id: method.method_id.as_str(),
        name: &method.name,
        request_type: &method.request_type,
        response_type: &method.response_type,
        stream: method.stream,
        operation_id: method.operation_id.as_str(),
        attributes: canonical_attributes_identity(&method.attributes),
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CanonicalAttributeIdentity<'a> {
    name: &'a str,
    syntax: &'a str,
}

fn canonical_attributes_identity(
    attributes: &[ContractAttribute],
) -> Vec<CanonicalAttributeIdentity<'_>> {
    let mut result = attributes
        .iter()
        .map(|attribute| CanonicalAttributeIdentity {
            name: &attribute.name,
            syntax: &attribute.syntax,
        })
        .collect::<Vec<_>>();
    // Sort by name then syntax to make the fingerprint stable across attribute
    // declaration order, which is not a semantic concern for identity.
    result.sort_by(|left, right| {
        left.name
            .cmp(right.name)
            .then(left.syntax.cmp(right.syntax))
    });
    result
}

// ---------------------------------------------------------------------------
// Helpers shared by from_artifact and from_full_artifact
// ---------------------------------------------------------------------------

/// Derives a stable artifact id from a contract artifact.
fn artifact_id_from_artifact(artifact: &ContractArtifact) -> String {
    format!("{}::{}", artifact.version.as_str(), artifact.producer.name)
}
