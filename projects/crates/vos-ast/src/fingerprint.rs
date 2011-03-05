//! Canonical [`schema_fingerprint`] for VOS catalogs.
//!
//! Fingerprints hash a stable semantic wire derived from durable catalog
//! identities (`TypeId`, `FieldId`, `MacroId`) and type / macro metadata. They
//! intentionally exclude publish counters (`Revisions`), tombstones, and virtual
//! slot indices so reorder-only evolution with preserved identities keeps the
//! same fingerprint.

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::FieldAttribute;
use crate::catalog::{CatalogSnapshot, TypeKind};
use crate::TypeExpr;

/// Lowercase hex SHA-256 of the canonical catalog semantic wire.
pub fn schema_fingerprint(catalog: &CatalogSnapshot) -> String {
    let wire = canonical_catalog_json(catalog);
    let digest = Sha256::digest(wire);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Build a catalog from `document` and return its [`schema_fingerprint`].
pub fn schema_fingerprint_from_document(
    document: &crate::Document,
) -> Result<String, String> {
    let catalog = crate::catalog::catalog_from_document(document)?;
    Ok(schema_fingerprint(&catalog))
}

fn canonical_catalog_json(catalog: &CatalogSnapshot) -> Vec<u8> {
    let mut types = catalog
        .types
        .iter()
        .map(CanonicalType::from)
        .collect::<Vec<_>>();
    types.sort_by_key(|entry| entry.id);
    let mut macros = catalog
        .macros
        .iter()
        .map(CanonicalMacro::from)
        .collect::<Vec<_>>();
    macros.sort_by_key(|entry| entry.id);
    serde_json::to_vec(&CanonicalCatalog { types, macros }).expect("canonical catalog serializes")
}

#[derive(Serialize)]
struct CanonicalCatalog {
    types: Vec<CanonicalType>,
    macros: Vec<CanonicalMacro>,
}

#[derive(Serialize)]
struct CanonicalType {
    id: u64,
    name: String,
    kind: TypeKind,
    fields: Vec<CanonicalField>,
}

#[derive(Serialize)]
struct CanonicalField {
    id: u64,
    name: String,
    ty: TypeExpr,
    attrs: Vec<FieldAttribute>,
}

impl CanonicalType {
    fn from(entry: &crate::catalog::TypeEntry) -> Self {
        let mut fields = entry.fields.iter().map(CanonicalField::from).collect::<Vec<_>>();
        fields.sort_by_key(|field| field.id);
        Self {
            id: entry.type_id.0,
            name: entry.name.clone(),
            kind: entry.kind,
            fields,
        }
    }
}

impl CanonicalField {
    fn from(slot: &crate::catalog::FieldSlot) -> Self {
        Self {
            id: slot.field_id.0,
            name: slot.current_name.clone(),
            ty: slot.ty.clone(),
            attrs: slot.attrs.clone(),
        }
    }
}

#[derive(Serialize)]
struct CanonicalMacro {
    id: u64,
    name: String,
    params: Vec<CanonicalMacroParam>,
    return_ty: Option<TypeExpr>,
}

#[derive(Serialize)]
struct CanonicalMacroParam {
    name: String,
    ty: TypeExpr,
}

impl CanonicalMacro {
    fn from(entry: &crate::catalog::MacroEntry) -> Self {
        let mut params = entry
            .params
            .iter()
            .map(|slot| CanonicalMacroParam {
                name: slot.name.clone(),
                ty: slot.ty.clone(),
            })
            .collect::<Vec<_>>();
        params.sort_by(|left, right| left.name.cmp(&right.name));
        Self {
            id: entry.macro_id.0,
            name: entry.name.clone(),
            params,
            return_ty: entry.return_ty.clone(),
        }
    }
}
