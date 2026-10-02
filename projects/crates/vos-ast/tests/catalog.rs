use std::collections::BTreeMap;

use vos_ast::catalog::{
    FieldId, FieldPath, RenameMap, TypeId, catalog_from_document, evolve_catalog,
};
use vos_ast::{BuiltinType, Document, Field, FieldAttribute, Item, Span, Table, TypeExpr};

fn sample_doc() -> Document {
    Document {
        namespace: None,
        items: vec![Item::Table(Table {
            name: "User".into(),
            fields: vec![
                Field {
                    name: "user_id".into(),
                    ty: TypeExpr::Builtin(BuiltinType::Uuid),
                    attrs: vec![FieldAttribute::Primary],
                    default: None,
                    span: Span::new(0, 1),
                },
                Field {
                    name: "user_name".into(),
                    ty: TypeExpr::Builtin(BuiltinType::Utf8),
                    attrs: vec![FieldAttribute::Unique],
                    default: None,
                    span: Span::new(1, 2),
                },
            ],
            span: Span::new(0, 2),
        })],
        source: String::new(),
    }
}

#[test]
fn assigns_stable_ids_and_slots() {
    let snap = catalog_from_document(&sample_doc()).unwrap();
    assert_eq!(snap.revisions.ddl, 1);
    assert_eq!(snap.types.len(), 1);
    let user = &snap.types[0];
    assert_eq!(user.type_id, TypeId(1));
    assert_eq!(user.fields[0].field_id, FieldId(1));
    assert_eq!(user.fields[0].virtual_field, 0);
    assert_eq!(user.fields[1].field_id, FieldId(2));
    assert_eq!(user.fields[1].virtual_field, 1);
    assert_eq!(user.fields[1].current_name, "user_name");
}

fn document(fields: &[(&str, TypeExpr)]) -> Document {
    Document {
        namespace: None,
        items: vec![Item::Table(Table {
            name: "User".into(),
            fields: fields
                .iter()
                .enumerate()
                .map(|(index, (name, ty))| Field {
                    name: (*name).into(),
                    ty: ty.clone(),
                    attrs: Vec::new(),
                    default: None,
                    span: Span::new(index, index + 1),
                })
                .collect(),
            span: Span::new(0, fields.len()),
        })],
        source: String::new(),
    }
}

#[test]
fn reorder_preserves_identity_and_slot() {
    let previous = catalog_from_document(&document(&[
        ("first", TypeExpr::Builtin(BuiltinType::I64)),
        ("second", TypeExpr::Builtin(BuiltinType::Bool)),
    ]))
    .unwrap();
    let next = evolve_catalog(
        &previous,
        &document(&[
            ("second", TypeExpr::Builtin(BuiltinType::Bool)),
            ("first", TypeExpr::Builtin(BuiltinType::I64)),
        ]),
        &RenameMap::default(),
    )
    .unwrap();
    let fields = &next.types[0].fields;
    assert_eq!(fields[0].current_name, "first");
    assert_eq!(fields[0].field_id, FieldId(1));
    assert_eq!(fields[0].virtual_field, 0);
    assert_eq!(fields[0].source_order, 1);
    assert_eq!(fields[1].current_name, "second");
    assert_eq!(fields[1].field_id, FieldId(2));
    assert_eq!(fields[1].virtual_field, 1);
}

#[test]
fn removed_identity_is_tombstoned_and_not_reused() {
    let previous = catalog_from_document(&document(&[
        ("first", TypeExpr::Builtin(BuiltinType::I64)),
        ("second", TypeExpr::Builtin(BuiltinType::Bool)),
    ]))
    .unwrap();
    let removed = evolve_catalog(
        &previous,
        &document(&[("first", TypeExpr::Builtin(BuiltinType::I64))]),
        &RenameMap::default(),
    )
    .unwrap();
    assert_eq!(removed.retired_fields.len(), 1);
    assert_eq!(removed.retired_fields[0].field_id, FieldId(2));
    let added = evolve_catalog(
        &removed,
        &document(&[
            ("first", TypeExpr::Builtin(BuiltinType::I64)),
            ("third", TypeExpr::Builtin(BuiltinType::Utf8)),
        ]),
        &RenameMap::default(),
    )
    .unwrap();
    assert_eq!(added.types[0].fields[1].field_id, FieldId(3));
    assert_eq!(added.types[0].fields[1].virtual_field, 2);
}

#[test]
fn explicit_field_rename_preserves_identity() {
    let previous = catalog_from_document(&document(&[(
        "old_name",
        TypeExpr::Builtin(BuiltinType::Utf8),
    )]))
    .unwrap();
    let mut fields = BTreeMap::new();
    fields.insert(
        FieldPath {
            type_name: "User".into(),
            field_name: "old_name".into(),
        },
        "new_name".into(),
    );
    let next = evolve_catalog(
        &previous,
        &document(&[("new_name", TypeExpr::Builtin(BuiltinType::Utf8))]),
        &RenameMap {
            types: BTreeMap::new(),
            fields,
        },
    )
    .unwrap();
    assert_eq!(next.types[0].fields[0].field_id, FieldId(1));
    assert_eq!(next.types[0].fields[0].virtual_field, 0);
}

#[test]
fn implicit_rename_allocates_new_identity() {
    let previous = catalog_from_document(&document(&[(
        "old_name",
        TypeExpr::Builtin(BuiltinType::Utf8),
    )]))
    .unwrap();
    let next = evolve_catalog(
        &previous,
        &document(&[("new_name", TypeExpr::Builtin(BuiltinType::Utf8))]),
        &RenameMap::default(),
    )
    .unwrap();
    assert_eq!(next.types[0].fields[0].field_id, FieldId(2));
    assert_eq!(next.retired_fields[0].field_id, FieldId(1));
}

#[test]
fn rejects_rename_that_matches_one_field_identity_twice() {
    let previous = catalog_from_document(&document(&[("old", TypeExpr::File)])).unwrap();
    let renames = RenameMap {
        fields: BTreeMap::from([(
            FieldPath { type_name: "User".into(), field_name: "old".into() },
            "new".into(),
        )]),
        ..RenameMap::default()
    };
    for fields in [
        vec![("old", TypeExpr::File), ("new", TypeExpr::File)],
        vec![("new", TypeExpr::File), ("old", TypeExpr::File)],
    ] {
        assert!(evolve_catalog(&previous, &document(&fields), &renames).is_err());
    }
}

#[test]
fn rejects_rename_that_matches_one_type_identity_twice() {
    let previous = catalog_from_document(&document(&[("id", TypeExpr::File)])).unwrap();
    let mut current = document(&[("id", TypeExpr::File)]);
    let mut renamed = current.items[0].clone();
    if let Item::Table(table) = &mut renamed { table.name = "Account".into(); }
    current.items.push(renamed);
    let renames = RenameMap {
        types: BTreeMap::from([("User".into(), "Account".into())]),
        ..RenameMap::default()
    };
    assert!(evolve_catalog(&previous, &current, &renames).is_err());
}

#[test]
fn explicit_type_rename_and_type_reorder_preserve_identity() {
    let mut initial = document(&[("id", TypeExpr::File)]);
    let mut second = initial.items[0].clone();
    if let Item::Table(table) = &mut second { table.name = "Account".into(); }
    initial.items.push(second);
    let previous = catalog_from_document(&initial).unwrap();
    let mut current = initial.clone();
    current.items.reverse();
    if let Item::Table(table) = &mut current.items[1] { table.name = "Person".into(); }
    let renames = RenameMap {
        types: BTreeMap::from([("User".into(), "Person".into())]),
        ..RenameMap::default()
    };
    let evolved = evolve_catalog(&previous, &current, &renames).unwrap();
    assert_eq!(evolved.types[0].type_id, TypeId(2));
    assert_eq!(evolved.types[1].type_id, TypeId(1));
    assert_eq!(evolved.types[1].fields[0].field_id, FieldId(1));
    assert!(evolved.retired_types.is_empty());
}

#[test]
fn historical_tombstones_do_not_advance_layout_epoch_again() {
    let initial = document(&[("id", TypeExpr::File), ("removed", TypeExpr::File)]);
    let previous = catalog_from_document(&initial).unwrap();
    let current = document(&[("id", TypeExpr::File)]);
    let removed = evolve_catalog(&previous, &current, &RenameMap::default()).unwrap();
    let repeated = evolve_catalog(&removed, &current, &RenameMap::default()).unwrap();
    assert_eq!(repeated.revisions.layout_epoch, removed.revisions.layout_epoch);
    assert_eq!(repeated.retired_fields, removed.retired_fields);
}

#[test]
fn exhausted_identity_slot_and_revision_counters_are_rejected() {
    let current = document(&[("id", TypeExpr::File)]);
    let previous = catalog_from_document(&current).unwrap();
    let mut exhausted = previous.clone();
    exhausted.types[0].type_id = TypeId(u64::MAX);
    assert!(evolve_catalog(&exhausted, &current, &RenameMap::default()).is_err());
    exhausted = previous.clone();
    exhausted.types[0].fields[0].field_id = FieldId(u64::MAX);
    assert!(evolve_catalog(&exhausted, &current, &RenameMap::default()).is_err());
    exhausted = previous.clone();
    exhausted.types[0].fields[0].virtual_field = u32::MAX;
    assert!(evolve_catalog(&exhausted, &current, &RenameMap::default()).is_err());
    exhausted = previous;
    exhausted.revisions.ddl = u64::MAX;
    assert!(evolve_catalog(&exhausted, &current, &RenameMap::default()).is_err());
}
