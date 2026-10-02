use vos_ast::catalog::{FieldId, TypeId, catalog_from_document};
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
