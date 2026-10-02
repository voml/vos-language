use vos_ast::Span;
use vos_ast::codes;
use vos_ast::expr::{Expr, FieldInit, Lambda, ProjItem, projection_result_names};

#[test]
fn field_init_shorthand_and_named() {
    let s = Span::new(0, 1);
    let sh = FieldInit::shorthand("user_id", s);
    assert!(sh.is_shorthand());
    let named = FieldInit::named("name", Expr::name("user_name", s), s);
    assert!(!named.is_shorthand());
    assert_eq!(named.name, "name");
}

#[test]
fn projection_names_detect_duplicate_with_spread() {
    let s = Span::new(0, 1);
    let items = vec![
        ProjItem::Star { span: s },
        ProjItem::Field(FieldInit::named("user_name", Expr::name("user_name", s), s)),
    ];
    let spread = vec!["user_id".into(), "user_name".into()];
    let err = projection_result_names(&items, Some(&spread)).unwrap_err();
    assert_eq!(err.0, "user_name");
    let _ = codes::PROJECTION_0004;
}

#[test]
fn projection_allows_new_name_beside_spread() {
    let s = Span::new(0, 1);
    let items = vec![
        ProjItem::Star { span: s },
        ProjItem::Field(FieldInit::named(
            "display_name",
            Expr::name("user_name", s),
            s,
        )),
    ];
    let spread = vec!["user_id".into(), "user_name".into()];
    let names = projection_result_names(&items, Some(&spread)).unwrap();
    assert_eq!(
        names,
        vec![
            "user_id".to_string(),
            "user_name".to_string(),
            "display_name".to_string()
        ]
    );
}

#[test]
fn sample_map_projection_ast_shape() {
    // User.map(x => x.{ user_id, name: user_name, avatar })
    let s = Span::new(0, 10);
    let x = Expr::name("x", s);
    let proj = Expr::StructProj {
        receiver: Box::new(x),
        items: vec![
            ProjItem::Field(FieldInit::shorthand("user_id", s)),
            ProjItem::Field(FieldInit::named("name", Expr::name("user_name", s), s)),
            ProjItem::Field(FieldInit::shorthand("avatar", s)),
        ],
        span: s,
    };
    let lambda = Expr::Lambda(Lambda {
        params: vec!["x".into()],
        body: Box::new(proj),
        span: s,
    });
    let map = Expr::Call {
        callee: Box::new(Expr::member(Expr::name("User", s), "map", s)),
        args: vec![lambda],
        span: s,
    };
    match map {
        Expr::Call { args, .. } => match &args[0] {
            Expr::Lambda(l) => match &*l.body {
                Expr::StructProj { items, .. } => {
                    assert_eq!(items.len(), 3);
                    assert_eq!(items[1].output_name(), Some("name"));
                }
                other => panic!("expected StructProj, got {other:?}"),
            },
            other => panic!("expected Lambda, got {other:?}"),
        },
        other => panic!("expected Call, got {other:?}"),
    }
}
