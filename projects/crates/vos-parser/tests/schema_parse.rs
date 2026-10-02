use vos_ast::{FieldAttribute, Item, TypeExpr};
use vos_parser::parse_document;

#[test]
fn parses_readme_user_table() {
    let doc = parse_document(
        r#"
            namespace demo::identity

            table User {
                @@user_id: uuid,
                @user_name: utf8,
                manager: &User? = null,
            }
            "#,
    )
    .unwrap();
    assert_eq!(doc.namespace.as_ref().unwrap().display(), "demo::identity");
    let table = doc.tables().next().unwrap();
    assert_eq!(table.name, "User");
    assert_eq!(table.fields.len(), 3);
    assert!(table.fields[0].is_primary());
    assert_eq!(table.fields[0].name, "user_id");
    assert!(table.fields[1].is_unique());
    assert!(matches!(
        table.fields[2].ty,
        TypeExpr::Optional(ref inner)
            if matches!(inner.as_ref(), TypeExpr::Reference(_))
    ));
}

#[test]
fn accepts_bracket_primary() {
    let doc = parse_document(
        r#"
            table User {
                [primary] id: utf8 = "",
                name: utf8 = "anonymous",
            }
            "#,
    )
    .unwrap();
    let id = &doc.tables().next().unwrap().fields[0];
    assert_eq!(id.name, "id");
    assert_eq!(id.attrs, vec![FieldAttribute::Primary]);
}

#[test]
fn rejects_missing_primary() {
    let err = parse_document("table Project { title: utf8 }").unwrap_err();
    assert!(err.errors.iter().any(|e| e.message.contains("primary key")));
}

#[test]
fn rejects_uuid_wrong_case() {
    let err = parse_document("table T { @@id: Uuid }").unwrap_err();
    assert!(err.errors.iter().any(|e| e.message.contains("Uuid")));
}

#[test]
fn rejects_unknown_reference_target() {
    let err = parse_document(
        r#"
            table A {
                @@id: uuid,
                other: &Missing,
            }
            "#,
    )
    .unwrap_err();
    assert!(err.errors.iter().any(|e| e.message.contains("Missing")));
}

#[test]
fn parses_class_enums_flags_and_obsolete() {
    let doc = parse_document(
        r#"
            enums Access {
                Yes = 1,
                No = 2,
            }

            flags Scope {
                Read = 0x01,
                Write = 0x10,
            }

            class LoginRequest {
                user_name: utf8,
                token: utf8,
            }

            table User {
                @@user_id: uuid,
            }

            obsolete field User.email;
            obsolete table Dealer;
            "#,
    )
    .unwrap();
    assert_eq!(doc.classes().count(), 1);
    assert!(doc.items.iter().any(|i| matches!(i, Item::Enums(_))));
    assert!(doc.items.iter().any(|i| matches!(i, Item::Flags(_))));
    assert!(doc.items.iter().any(|i| matches!(i, Item::Obsolete(_))));
}
