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

/// A field name in a previous catalog snapshot.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FieldPath {
    /// Previous owner type name.
    pub type_name: String,
    /// Previous field name.
    pub field_name: String,
}

/// Explicit schema identity changes between two catalog snapshots.
///
/// Names are never matched by similarity. A rename must be recorded here or
/// it is treated as removal of the old identity and creation of a new one.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenameMap {
    /// Previous type name to current type name.
    #[serde(default)]
    pub types: BTreeMap<String, String>,
    /// Previous field path to current field name.
    #[serde(default)]
    pub fields: BTreeMap<FieldPath, String>,
}

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

/// One live field in the virtual slot map.
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
    /// Removed type identities retained so their IDs are never reused.
    #[serde(default)]
    pub retired_types: Vec<RetiredType>,
    /// Removed field identities retained so their IDs and slots are never reused.
    #[serde(default)]
    pub retired_fields: Vec<RetiredField>,
}

/// A type identity that is no longer live.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetiredType {
    /// Durable type identity.
    pub type_id: TypeId,
    /// Last published name.
    pub last_name: String,
    /// Last published kind.
    pub kind: TypeKind,
}

/// A field identity that is no longer live.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetiredField {
    /// Durable owner type identity.
    pub type_id: TypeId,
    /// Durable field identity.
    pub field_id: FieldId,
    /// Virtual slot that must not be reused.
    pub virtual_field: VirtualFieldIndex,
    /// Last published name.
    pub last_name: String,
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
        retired_types: Vec::new(),
        retired_fields: Vec::new(),
    })
}

/// Evolve a catalog while preserving explicitly matched identities.
///
/// Existing names are matched exactly. Type and field renames are matched only
/// through `renames`. Reordering changes source order but never changes a live
/// field's `FieldId` or `virtual_field`. Removed identities are retained as
/// tombstones and are excluded from future allocation.
pub fn evolve_catalog(
    previous: &CatalogSnapshot,
    document: &Document,
    renames: &RenameMap,
) -> Result<CatalogSnapshot, String> {
    let current = catalog_declarations(document)?;
    validate_rename_targets(previous, &current, renames)?;

    let mut next_type_id = next_type_id(previous);
    let mut next_field_id = next_field_id(previous);
    let mut matched_type_ids = BTreeMap::<TypeId, ()>::new();
    let mut matched_field_ids = BTreeMap::<FieldId, ()>::new();
    let mut types = Vec::with_capacity(current.len());
    let mut retired_types = previous.retired_types.clone();
    let mut retired_fields = previous.retired_fields.clone();

    for (current_name, kind, fields) in current {
        let previous_type = find_previous_type(previous, &current_name, renames)?;
        let (type_id, old_name, old_fields) = if let Some(entry) = previous_type {
            if entry.kind != kind {
                return Err(format!(
                    "type `{current_name}` changed kind from {:?} to {:?}",
                    entry.kind, kind
                ));
            }
            matched_type_ids.insert(entry.type_id, ());
            (entry.type_id, entry.name.clone(), entry.fields.clone())
        } else {
            let id = TypeId(next_type_id);
            next_type_id = next_type_id.saturating_add(1);
            (id, current_name.clone(), Vec::new())
        };

        let mut live_fields = Vec::with_capacity(fields.len());
        let mut matched_old_fields = BTreeMap::<FieldId, ()>::new();
        let mut next_slot = next_virtual_slot(previous, type_id, &old_fields);
        for (source_order, field) in fields.iter().enumerate() {
            let previous_field = find_previous_field(
                &old_name,
                &old_fields,
                field,
                renames,
            )?;
            let slot = if let Some(old_field) = previous_field {
                matched_field_ids.insert(old_field.field_id, ());
                matched_old_fields.insert(old_field.field_id, ());
                FieldSlot {
                    field_id: old_field.field_id,
                    virtual_field: old_field.virtual_field,
                    current_name: field.name.clone(),
                    source_order: source_order as u32,
                    ty: field.ty.clone(),
                    attrs: field.attrs.clone(),
                }
            } else {
                let slot = FieldSlot {
                    field_id: FieldId(next_field_id),
                    virtual_field: next_slot,
                    current_name: field.name.clone(),
                    source_order: source_order as u32,
                    ty: field.ty.clone(),
                    attrs: field.attrs.clone(),
                };
                next_field_id = next_field_id.saturating_add(1);
                next_slot = next_slot.saturating_add(1);
                slot
            };
            live_fields.push(slot);
        }

        for old_field in old_fields {
            if !matched_old_fields.contains_key(&old_field.field_id) {
                retired_fields.push(RetiredField {
                    type_id,
                    field_id: old_field.field_id,
                    virtual_field: old_field.virtual_field,
                    last_name: old_field.current_name,
                });
            }
        }

        live_fields.sort_by_key(|field| field.virtual_field);
        types.push(TypeEntry {
            type_id,
            name: current_name,
            kind,
            fields: live_fields,
        });
    }

    for old_type in &previous.types {
        if !matched_type_ids.contains_key(&old_type.type_id) {
            retired_types.push(RetiredType {
                type_id: old_type.type_id,
                last_name: old_type.name.clone(),
                kind: old_type.kind,
            });
            for old_field in &old_type.fields {
                if !matched_field_ids.contains_key(&old_field.field_id) {
                    retired_fields.push(RetiredField {
                        type_id: old_type.type_id,
                        field_id: old_field.field_id,
                        virtual_field: old_field.virtual_field,
                        last_name: old_field.current_name.clone(),
                    });
                }
            }
        }
    }

    deduplicate_tombstones(&mut retired_types, &mut retired_fields);
    let changed_layout = layout_changed(previous, &types, &retired_fields);
    Ok(CatalogSnapshot {
        revisions: Revisions {
            ddl: previous.revisions.ddl.saturating_add(1),
            semantic: previous.revisions.semantic.saturating_add(1),
            layout_epoch: if changed_layout {
                previous.revisions.layout_epoch.saturating_add(1)
            } else {
                previous.revisions.layout_epoch
            },
        },
        types,
        retired_types,
        retired_fields,
    })
}

fn catalog_declarations(
    document: &Document,
) -> Result<Vec<(String, TypeKind, Vec<Field>)>, String> {
    let mut names = BTreeMap::<String, ()>::new();
    let mut declarations = Vec::new();
    for item in &document.items {
        let (name, kind, fields) = match item {
            Item::Table(table) => (&table.name, TypeKind::Table, &table.fields),
            Item::Class(class) => (&class.name, TypeKind::Class, &class.fields),
            _ => continue,
        };
        if names.insert(name.clone(), ()).is_some() {
            return Err(format!("duplicate type `{name}`"));
        }
        let mut field_names = BTreeMap::<String, ()>::new();
        for field in fields {
            if field_names.insert(field.name.clone(), ()).is_some() {
                return Err(format!("duplicate field `{}` on `{name}`", field.name));
            }
        }
        declarations.push((name.clone(), kind, fields.clone()));
    }
    Ok(declarations)
}

fn validate_rename_targets(
    previous: &CatalogSnapshot,
    current: &[(String, TypeKind, Vec<Field>)],
    renames: &RenameMap,
) -> Result<(), String> {
    let current_types = current
        .iter()
        .map(|(name, _, _)| (name.as_str(), ()))
        .collect::<BTreeMap<_, _>>();
    for (old_name, new_name) in &renames.types {
        if previous.types.iter().all(|entry| entry.name != *old_name) {
            return Err(format!("type rename source `{old_name}` does not exist"));
        }
        if !current_types.contains_key(new_name.as_str()) {
            return Err(format!("type rename target `{new_name}` does not exist"));
        }
    }
    for (path, new_name) in &renames.fields {
        let old_type = previous
            .types
            .iter()
            .find(|entry| entry.name == path.type_name)
            .ok_or_else(|| format!("field rename owner `{}` does not exist", path.type_name))?;
        if old_type
            .fields
            .iter()
            .all(|field| field.current_name != path.field_name)
        {
            return Err(format!(
                "field rename source `{}.{}` does not exist",
                path.type_name, path.field_name
            ));
        }
        let current_type_name = renames
            .types
            .get(&path.type_name)
            .map_or(path.type_name.as_str(), String::as_str);
        let current_type = current
            .iter()
            .find(|(name, _, _)| name == current_type_name)
            .ok_or_else(|| format!("field rename owner `{current_type_name}` does not exist"))?;
        if current_type
            .2
            .iter()
            .all(|field| field.name != *new_name)
        {
            return Err(format!("field rename target `{new_name}` does not exist"));
        }
    }
    Ok(())
}

fn find_previous_type<'a>(
    previous: &'a CatalogSnapshot,
    current_name: &str,
    renames: &RenameMap,
) -> Result<Option<&'a TypeEntry>, String> {
    let direct = previous.types.iter().find(|entry| entry.name == current_name);
    let renamed = renames
        .types
        .iter()
        .filter(|(_, target)| target.as_str() == current_name)
        .map(|(source, _)| source.as_str())
        .collect::<Vec<_>>();
    if renamed.len() > 1 {
        return Err(format!("multiple type identities target `{current_name}`"));
    }
    if direct.is_some() && !renamed.is_empty() {
        return Err(format!("type `{current_name}` has direct and renamed identities"));
    }
    Ok(direct.or_else(|| renamed.first().and_then(|name| previous.types.iter().find(|entry| entry.name == *name))))
}

fn find_previous_field<'a>(
    old_type_name: &str,
    old_fields: &'a [FieldSlot],
    current: &Field,
    renames: &RenameMap,
) -> Result<Option<&'a FieldSlot>, String> {
    let direct = old_fields
        .iter()
        .find(|field| field.current_name == current.name);
    let renamed = renames
        .fields
        .iter()
        .filter(|(path, target)| path.type_name == old_type_name && target.as_str() == current.name)
        .map(|(path, _)| path.field_name.as_str())
        .collect::<Vec<_>>();
    if renamed.len() > 1 {
        return Err(format!(
            "multiple field identities target `{old_type_name}.{} `",
            current.name
        ));
    }
    if direct.is_some() && !renamed.is_empty() {
        return Err(format!(
            "field `{old_type_name}.{} ` has direct and renamed identities",
            current.name
        ));
    }
    Ok(direct.or_else(|| renamed.first().and_then(|name| old_fields.iter().find(|field| field.current_name == *name))))
}

fn next_type_id(snapshot: &CatalogSnapshot) -> u64 {
    snapshot
        .types
        .iter()
        .map(|entry| entry.type_id.0)
        .chain(snapshot.retired_types.iter().map(|entry| entry.type_id.0))
        .max()
        .unwrap_or(0)
        .saturating_add(1)
}

fn next_field_id(snapshot: &CatalogSnapshot) -> u64 {
    snapshot
        .types
        .iter()
        .flat_map(|entry| entry.fields.iter().map(|field| field.field_id.0))
        .chain(snapshot.retired_fields.iter().map(|field| field.field_id.0))
        .max()
        .unwrap_or(0)
        .saturating_add(1)
}

fn next_virtual_slot(
    snapshot: &CatalogSnapshot,
    type_id: TypeId,
    live_fields: &[FieldSlot],
) -> VirtualFieldIndex {
    live_fields
        .iter()
        .map(|field| field.virtual_field)
        .chain(
            snapshot
                .retired_fields
                .iter()
                .filter(|field| field.type_id == type_id)
                .map(|field| field.virtual_field),
        )
        .max()
        .map_or(0, |slot| slot.saturating_add(1))
}

fn deduplicate_tombstones(types: &mut Vec<RetiredType>, fields: &mut Vec<RetiredField>) {
    types.sort_by_key(|entry| entry.type_id);
    types.dedup_by_key(|entry| entry.type_id);
    fields.sort_by_key(|entry| entry.field_id);
    fields.dedup_by_key(|entry| entry.field_id);
}

fn layout_changed(
    previous: &CatalogSnapshot,
    current: &[TypeEntry],
    retired_fields: &[RetiredField],
) -> bool {
    if !retired_fields.is_empty() {
        return true;
    }
    previous.types.iter().any(|old| {
        current.iter().find(|entry| entry.type_id == old.type_id).is_some_and(|new| {
            old.fields.iter().any(|old_field| {
                new.fields.iter().find(|field| field.field_id == old_field.field_id).is_some_and(|new_field| old_field.ty != new_field.ty)
            })
        })
    }) || current.iter().any(|entry| entry.fields.iter().any(|field| {
        previous.types.iter().all(|old| old.fields.iter().all(|old_field| old_field.field_id != field.field_id))
    }))
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
