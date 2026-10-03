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
fn preserves_using_source_items() {
    let doc = parse_document(
        r#"
            namespace demo;
            using shared::UserId;

            table User {
                @@id: uuid,
            }
            "#,
    )
    .unwrap();

    let using = doc.usings().next().expect("using item");
    assert_eq!(using.display(), "shared::UserId");
    assert!(using.span.end > using.span.start);
    assert!(matches!(doc.items.first(), Some(Item::Using(_))));
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

#[test]
fn parses_durable_macro_item() {
    let doc = parse_document(
        r#"
            table User {
                @@user_id: uuid,
                user_name: utf8,
            }

            macro public_name(value: utf8) -> utf8 {
                value
            }
            "#,
    )
    .unwrap();
    let macro_item = doc
        .items
        .iter()
        .find_map(|item| match item {
            Item::Macro(macro_def) => Some(macro_def),
            _ => None,
        })
        .expect("macro item");
    assert_eq!(macro_item.name, "public_name");
    assert_eq!(macro_item.params.len(), 1);
    assert_eq!(macro_item.params[0].name, "value");
}

#[test]
fn parses_seed_blog_macro_with_inserts() {
    let doc = parse_document(
        r#"
            table User {
                @@user_id: uuid,
                user_name: utf8,
                active: bool,
            }

            table Post {
                @@post_id: uuid,
                author: &User,
                title: utf8,
                published: bool,
            }

            macro seed_blog() -> unit {
                User {
                    user_id: "550e8400-e29b-41d4-a716-446655440000",
                    user_name: "ada",
                    active: true,
                }.insert()
                User {
                    user_id: "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
                    user_name: "linus",
                    active: true,
                }.insert()
                Post {
                    post_id: "11111111-1111-4111-8111-111111111101",
                    author: "550e8400-e29b-41d4-a716-446655440000",
                    title: "Ada post",
                    published: true,
                }.insert()
                Post {
                    post_id: "22222222-2222-4222-8222-222222222202",
                    author: "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
                    title: "Linus post",
                    published: true,
                }.insert()
            }
            "#,
    )
    .unwrap();
    let macro_item = doc
        .items
        .iter()
        .find_map(|item| match item {
            Item::Macro(macro_def) => Some(macro_def),
            _ => None,
        })
        .expect("seed_blog macro");
    assert_eq!(macro_item.name, "seed_blog");
    assert!(macro_item.return_ty.is_some());
}
