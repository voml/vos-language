//! Initial field-identity catalog IR (language contract).
//!
//! Field catalog IR: `FieldId`, virtual slots, layout epoch.
//! Hosts such as YYDB may persist a richer catalog blob; the **assignment
//! algorithm** for a fresh document (type/field ids, virtual slots, revisions)
//! must match this module so conformance goldens stay host-independent.

use crate::expr::{FnDecl, FnKind};
use crate::{Document, Field, FieldAttribute, Item, TypeExpr};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Stable type identity inside one catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TypeId(pub u64);

/// Stable field identity across rename / reorder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FieldId(pub u64);

/// Stable macro identity across rename / reorder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MacroId(pub u64);

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
    /// Previous macro name to current macro name.
    #[serde(default)]
    pub macros: BTreeMap<String, String>,
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

/// One parameter slot in a durable macro signature.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacroParamSlot {
    /// Parameter name.
    pub name: String,
    /// Parameter type.
    pub ty: TypeExpr,
    /// Source declaration order.
    pub source_order: u32,
}

/// One durable `macro` entry in the catalog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacroEntry {
    /// Durable macro identity.
    pub macro_id: MacroId,
    /// Current macro name.
    pub name: String,
    /// Parameters in source order.
    pub params: Vec<MacroParamSlot>,
    /// Optional return type.
    pub return_ty: Option<TypeExpr>,
    /// Source declaration order among macros.
    pub source_order: u32,
}

/// Deterministic catalog snapshot for conformance (`*.catalog.json`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatalogSnapshot {
    /// Publish counters after initial build.
    pub revisions: Revisions,
    /// Types in document order.
    pub types: Vec<TypeEntry>,
    /// Removed type identities retained so their IDs are never reused.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retired_types: Vec<RetiredType>,
    /// Removed field identities retained so their IDs and slots are never reused.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retired_fields: Vec<RetiredField>,
    /// Live durable macros in document order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub macros: Vec<MacroEntry>,
    /// Removed macro identities retained so their IDs are never reused.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retired_macros: Vec<RetiredMacro>,
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

/// A macro identity that is no longer live.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetiredMacro {
    /// Durable macro identity.
    pub macro_id: MacroId,
    /// Last published name.
    pub last_name: String,
}

/// Build the initial catalog from a parsed document.
///
/// Allocation rules (locked for goldens):
/// - `TypeId` / `FieldId` counters start at `1` and increase in document order.
/// - Tables, classes, and durable `macro` items are catalogued; enums / flags /
///   obsolete are skipped.
/// - `MacroId` counters start at `1` and increase in document order among macros.
/// - Each field’s first `VirtualFieldIndex` equals its source order.
/// - Initial publish sets `ddl = 1`, `semantic = 1`, `layout_epoch = 0`.
pub fn catalog_from_document(document: &Document) -> Result<CatalogSnapshot, String> {
    let mut next_type_id = 1u64;
    let mut next_field_id = 1u64;
    let mut next_macro_id = 1u64;
    let mut types = Vec::new();
    let mut macros = Vec::new();
    let mut seen_names = BTreeMap::<String, ()>::new();
    let mut seen_macro_names = BTreeMap::<String, ()>::new();
    let mut macro_order = 0u32;

    for item in &document.items {
        match item {
            Item::Macro(macro_def) => {
                if macro_def.kind != FnKind::Macro {
                    return Err(format!(
                        "document item `{}` must be a durable macro",
                        macro_def.name
                    ));
                }
                if seen_macro_names.insert(macro_def.name.clone(), ()).is_some() {
                    return Err(format!("duplicate macro `{}`", macro_def.name));
                }
                if seen_names.contains_key(&macro_def.name) {
                    return Err(format!(
                        "macro `{}` collides with an existing type name",
                        macro_def.name
                    ));
                }
                let macro_id = MacroId(next_macro_id);
                next_macro_id += 1;
                macros.push(macro_entry_from_decl(macro_def, macro_id, macro_order));
                macro_order += 1;
            }
            Item::Table(table) => {
                if seen_names.insert(table.name.clone(), ()).is_some() {
                    return Err(format!("duplicate type `{}`", table.name));
                }
                if seen_macro_names.contains_key(&table.name) {
                    return Err(format!(
                        "table `{}` collides with an existing macro name",
                        table.name
                    ));
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
                if seen_macro_names.contains_key(&class.name) {
                    return Err(format!(
                        "class `{}` collides with an existing macro name",
                        class.name
                    ));
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
        macros,
        retired_macros: Vec::new(),
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
    let current_macros = catalog_macro_declarations(document)?;
    validate_name_collisions(&current, &current_macros)?;
    validate_rename_targets(previous, &current, &current_macros, renames)?;

    let mut next_type_id = next_type_id(previous)?;
    let mut next_field_id = next_field_id(previous)?;
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
            if matched_type_ids.insert(entry.type_id, ()).is_some() {
                return Err(format!("type identity {} matched more than once", entry.type_id.0));
            }
            (entry.type_id, entry.name.clone(), entry.fields.clone())
        } else {
            let id = TypeId(next_type_id);
            next_type_id = next_type_id.checked_add(1).ok_or("type identity exhausted")?;
            (id, current_name.clone(), Vec::new())
        };

        let mut live_fields = Vec::with_capacity(fields.len());
        let mut matched_old_fields = BTreeMap::<FieldId, ()>::new();
        let mut next_slot = next_virtual_slot(previous, type_id, &old_fields)?;
        for (source_order, field) in fields.iter().enumerate() {
            let previous_field = find_previous_field(
                &old_name,
                &old_fields,
                field,
                renames,
            )?;
            let slot = if let Some(old_field) = previous_field {
                if matched_field_ids.insert(old_field.field_id, ()).is_some() {
                    return Err(format!("field identity {} matched more than once", old_field.field_id.0));
                }
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
                next_field_id = next_field_id.checked_add(1).ok_or("field identity exhausted")?;
                next_slot = next_slot.checked_add(1).ok_or("virtual slot exhausted")?;
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

    let mut next_macro_id = next_macro_id(previous)?;
    let mut matched_macro_ids = BTreeMap::<MacroId, ()>::new();
    let mut macros = Vec::with_capacity(current_macros.len());
    let mut retired_macros = previous.retired_macros.clone();
    let mut macro_order = 0u32;

    for macro_def in current_macros {
        let previous_macro = find_previous_macro(previous, &macro_def.name, renames)?;
        let macro_id = if let Some(entry) = previous_macro {
            if matched_macro_ids.insert(entry.macro_id, ()).is_some() {
                return Err(format!(
                    "macro identity {} matched more than once",
                    entry.macro_id.0
                ));
            }
            entry.macro_id
        } else {
            let id = MacroId(next_macro_id);
            next_macro_id = next_macro_id.checked_add(1).ok_or("macro identity exhausted")?;
            id
        };
        macros.push(macro_entry_from_decl(&macro_def, macro_id, macro_order));
        macro_order += 1;
    }

    for old_macro in &previous.macros {
        if !matched_macro_ids.contains_key(&old_macro.macro_id) {
            retired_macros.push(RetiredMacro {
                macro_id: old_macro.macro_id,
                last_name: old_macro.name.clone(),
            });
        }
    }

    deduplicate_macro_tombstones(&mut retired_macros);
    let changed_layout = layout_changed(previous, &types, &retired_fields);
    Ok(CatalogSnapshot {
        revisions: Revisions {
            ddl: previous.revisions.ddl.checked_add(1).ok_or("DDL revision exhausted")?,
            semantic: previous.revisions.semantic.checked_add(1).ok_or("semantic revision exhausted")?,
            layout_epoch: if changed_layout {
                previous.revisions.layout_epoch.checked_add(1).ok_or("layout epoch exhausted")?
            } else {
                previous.revisions.layout_epoch
            },
        },
        types,
        retired_types,
        retired_fields,
        macros,
        retired_macros,
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

fn catalog_macro_declarations(document: &Document) -> Result<Vec<FnDecl>, String> {
    let mut names = BTreeMap::<String, ()>::new();
    let mut declarations = Vec::new();
    for item in &document.items {
        if let Item::Macro(macro_def) = item {
            if macro_def.kind != FnKind::Macro {
                return Err(format!(
                    "document item `{}` must be a durable macro",
                    macro_def.name
                ));
            }
            if names.insert(macro_def.name.clone(), ()).is_some() {
                return Err(format!("duplicate macro `{}`", macro_def.name));
            }
            let mut param_names = BTreeMap::<String, ()>::new();
            for param in &macro_def.params {
                if param_names.insert(param.name.clone(), ()).is_some() {
                    return Err(format!(
                        "duplicate parameter `{}` on macro `{}`",
                        param.name,
                        macro_def.name
                    ));
                }
            }
            declarations.push(macro_def.clone());
        }
    }
    Ok(declarations)
}

fn validate_name_collisions(
    current: &[(String, TypeKind, Vec<Field>)],
    current_macros: &[FnDecl],
) -> Result<(), String> {
    for (name, _, _) in current {
        if current_macros.iter().any(|macro_def| macro_def.name == *name) {
            return Err(format!("type `{name}` collides with macro `{name}`"));
        }
    }
    Ok(())
}

fn validate_rename_targets(
    previous: &CatalogSnapshot,
    current: &[(String, TypeKind, Vec<Field>)],
    current_macros: &[FnDecl],
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
    let current_macro_names = current_macros
        .iter()
        .map(|macro_def| (macro_def.name.as_str(), ()))
        .collect::<BTreeMap<_, _>>();
    for (old_name, new_name) in &renames.macros {
        if previous.macros.iter().all(|entry| entry.name != *old_name) {
            return Err(format!("macro rename source `{old_name}` does not exist"));
        }
        if !current_macro_names.contains_key(new_name.as_str()) {
            return Err(format!("macro rename target `{new_name}` does not exist"));
        }
    }
    Ok(())
}

fn find_previous_macro<'a>(
    previous: &'a CatalogSnapshot,
    current_name: &str,
    renames: &RenameMap,
) -> Result<Option<&'a MacroEntry>, String> {
    let direct = previous.macros.iter().find(|entry| entry.name == current_name);
    let renamed = renames
        .macros
        .iter()
        .filter(|(_, target)| target.as_str() == current_name)
        .map(|(source, _)| source.as_str())
        .collect::<Vec<_>>();
    if renamed.len() > 1 {
        return Err(format!("multiple macro identities target `{current_name}`"));
    }
    if direct.is_some() && !renamed.is_empty() {
        return Err(format!("macro `{current_name}` has direct and renamed identities"));
    }
    Ok(direct.or_else(|| {
        renamed
            .first()
            .and_then(|name| previous.macros.iter().find(|entry| entry.name == *name))
    }))
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

fn next_type_id(snapshot: &CatalogSnapshot) -> Result<u64, String> {
    snapshot
        .types
        .iter()
        .map(|entry| entry.type_id.0)
        .chain(snapshot.retired_types.iter().map(|entry| entry.type_id.0))
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(|| "type identity exhausted".into())
}

fn next_macro_id(snapshot: &CatalogSnapshot) -> Result<u64, String> {
    snapshot
        .macros
        .iter()
        .map(|entry| entry.macro_id.0)
        .chain(snapshot.retired_macros.iter().map(|entry| entry.macro_id.0))
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(|| "macro identity exhausted".into())
}

fn next_field_id(snapshot: &CatalogSnapshot) -> Result<u64, String> {
    snapshot
        .types
        .iter()
        .flat_map(|entry| entry.fields.iter().map(|field| field.field_id.0))
        .chain(snapshot.retired_fields.iter().map(|field| field.field_id.0))
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(|| "field identity exhausted".into())
}

fn next_virtual_slot(
    snapshot: &CatalogSnapshot,
    type_id: TypeId,
    live_fields: &[FieldSlot],
) -> Result<VirtualFieldIndex, String> {
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
        .map_or(Ok(0), |slot| slot.checked_add(1).ok_or_else(|| "virtual slot exhausted".into()))
}

fn deduplicate_tombstones(types: &mut Vec<RetiredType>, fields: &mut Vec<RetiredField>) {
    types.sort_by_key(|entry| entry.type_id);
    types.dedup_by_key(|entry| entry.type_id);
    fields.sort_by_key(|entry| entry.field_id);
    fields.dedup_by_key(|entry| entry.field_id);
}

fn deduplicate_macro_tombstones(macros: &mut Vec<RetiredMacro>) {
    macros.sort_by_key(|entry| entry.macro_id);
    macros.dedup_by_key(|entry| entry.macro_id);
}

fn macro_entry_from_decl(macro_def: &FnDecl, macro_id: MacroId, source_order: u32) -> MacroEntry {
    MacroEntry {
        macro_id,
        name: macro_def.name.clone(),
        params: macro_def
            .params
            .iter()
            .enumerate()
            .map(|(order, param)| MacroParamSlot {
                name: param.name.clone(),
                ty: param.ty.clone(),
                source_order: order as u32,
            })
            .collect(),
        return_ty: macro_def.return_ty.clone(),
        source_order,
    }
}

fn layout_changed(
    previous: &CatalogSnapshot,
    current: &[TypeEntry],
    retired_fields: &[RetiredField],
) -> bool {
    if retired_fields != previous.retired_fields || current.len() != previous.types.len() {
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
