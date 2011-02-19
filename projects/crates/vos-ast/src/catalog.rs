//! Initial field-identity catalog IR (language contract).
//!
//! Field catalog IR: `FieldId`, virtual slots, layout epoch.
//! Hosts such as YYDB may persist a richer catalog blob; the **assignment
//! algorithm** for a fresh document (type/field ids, virtual slots, revisions)
//! must match this module so conformance goldens stay host-independent.

use crate::{Document, Field, FieldAttribute, Item, TypeExpr};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Stable type identity inside one catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TypeId(pub u64);

/// Stable field identity across rename / reorder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FieldId(pub u64);

/// Virtual field slot — assigned once, never reused.
pub type VirtualFieldIndex = u32;

/// Catalog publish counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Revisions {
    /// Canonical DDL / catalog publish generation.
    pub ddl: u64,
    /// Observable type / constraint semantics generation.
    pub semantic: u64,
    /// Physical row encoding generation.
    pub layout_epoch: u64,
}

/// Kind of named type carrying fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TypeKind {
    /// Persistence `table`.
    Table,
    /// Non-persistent `class`.
    Class,
}

/// One live field in the virtual slot map (initial catalog has no tombstones).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldSlot {
    /// Durable identity.
    pub field_id: FieldId,
    /// Virtual slot index (== source order on first publish).
    pub virtual_field: VirtualFieldIndex,
    /// Current source / display name.
    pub current_name: String,
    /// Source declaration order.
    pub source_order: u32,
    /// VOS type expression.
    pub ty: TypeExpr,
    /// Primary / unique attributes.
    pub attrs: Vec<FieldAttribute>,
}

/// One table/class entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeEntry {
    /// Durable type id.
    pub type_id: TypeId,
    /// Current type name.
    pub name: String,
    /// Table vs class.
    pub kind: TypeKind,
    /// Live fields in virtual-slot order.
    pub fields: Vec<FieldSlot>,
}

/// Deterministic catalog snapshot for conformance (`*.catalog.json`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatalogSnapshot {
    /// Publish counters after initial build.
    pub revisions: Revisions,
    /// Types in document order.
    pub types: Vec<TypeEntry>,
}

/// Build the initial catalog from a parsed document.
///
/// Allocation rules (locked for goldens):
/// - `TypeId` / `FieldId` counters start at `1` and increase in document order.
/// - Tables and classes are catalogued; enums / flags / obsolete are skipped.
/// - Each field’s first `VirtualFieldIndex` equals its source order.
/// - Initial publish sets `ddl = 1`, `semantic = 1`, `layout_epoch = 0`.
pub fn catalog_from_document(document: &Document) -> Result<CatalogSnapshot, String> {
    let mut next_type_id = 1u64;
    let mut next_field_id = 1u64;
    let mut types = Vec::new();
    let mut seen_names = BTreeMap::<String, ()>::new();

    for item in &document.items {
        match item {
            Item::Table(table) => {
                if seen_names.insert(table.name.clone(), ()).is_some() {
                    return Err(format!("duplicate type `{}`", table.name));
                }
                let type_id = TypeId(next_type_id);
                next_type_id += 1;
                let fields = assign_fields(&table.fields, &mut next_field_id, &table.name)?;
                types.push(TypeEntry {
                    type_id,
                    name: table.name.clone(),
                    kind: TypeKind::Table,
                    fields,
                });
            }
            Item::Class(class) => {
                if seen_names.insert(class.name.clone(), ()).is_some() {
                    return Err(format!("duplicate type `{}`", class.name));
                }
                let type_id = TypeId(next_type_id);
                next_type_id += 1;
                let fields = assign_fields(&class.fields, &mut next_field_id, &class.name)?;
                types.push(TypeEntry {
                    type_id,
                    name: class.name.clone(),
                    kind: TypeKind::Class,
                    fields,
                });
            }
            _ => {}
        }
    }

    Ok(CatalogSnapshot {
        revisions: Revisions {
            ddl: 1,
            semantic: 1,
            layout_epoch: 0,
        },
        types,
    })
}

fn assign_fields(
    fields: &[Field],
    next_field_id: &mut u64,
    owner: &str,
) -> Result<Vec<FieldSlot>, String> {
    let mut out = Vec::with_capacity(fields.len());
    let mut names = BTreeMap::<String, ()>::new();
    for (order, field) in fields.iter().enumerate() {
        if names.insert(field.name.clone(), ()).is_some() {
            return Err(format!("duplicate field `{}` on `{owner}`", field.name));
        }
        let field_id = FieldId(*next_field_id);
        *next_field_id += 1;
        out.push(FieldSlot {
            field_id,
            virtual_field: order as VirtualFieldIndex,
            current_name: field.name.clone(),
            source_order: order as u32,
            ty: field.ty.clone(),
            attrs: field.attrs.clone(),
        });
    }
    Ok(out)
}
