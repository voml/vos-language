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
