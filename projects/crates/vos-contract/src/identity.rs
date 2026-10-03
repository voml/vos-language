use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{
    AttributeContract, CanonicalType, ContractDiagnostic, SchemaProjection, TypeContractKind,
};

/// Version of the explicit durable identity manifest.
pub const IDENTITY_MANIFEST_VERSION: &str = "vos-identity-manifest-v0";

/// A reviewable identity assignment for one schema projection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IdentityManifest {
    /// Manifest format version.
    pub format_version: String,
    /// Type identities. Array order is not semantic.
    pub types: Vec<TypeIdentity>,
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdentityBoundProjection {
    /// Identity manifest version.
    pub manifest_version: String,
    /// Bound types in projection source order.
    pub types: Vec<BoundTypeContract>,
}

/// A projected type with a durable type ID.
#[derive(Clone, Debug, PartialEq, Eq)]
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
#[derive(Clone, Debug, PartialEq, Eq)]
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
