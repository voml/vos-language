use vos_ast::expr::{Expr, Let, PathSep, Stmt};
use vos_parser::parse_program;

#[test]
fn rejects_macro_in_operation_program() {
    let err = parse_program(
        r#"
            macro public_name(value: utf8) -> utf8 {
                value
            }
            public_name
            "#,
    )
    .unwrap_err();
    assert!(
        err.errors
            .iter()
            .any(|e| e.code.as_deref() == Some("VOS-DDL-SESSION-REQUIRED"))
    );
}

#[test]
fn parses_name_colon_projection() {
    let program = parse_program(
        r#"
            let users = User.map(x => x.{
                user_id,
                name: user_name,
            }).collect()
            users
            "#,
    )
    .unwrap();
    match &program.statements[0] {
        Stmt::Let(Let { value, .. }) => {
            let found = contains_struct_proj(value);
            assert!(found, "expected StructProj in AST: {value:?}");
        }
        other => panic!("{other:?}"),
    }
}

fn contains_struct_proj(expr: &Expr) -> bool {
    match expr {
        Expr::StructProj { items, .. } => items.iter().any(|i| i.output_name() == Some("name")),
        Expr::Call { callee, args, .. } => {
            contains_struct_proj(callee) || args.iter().any(contains_struct_proj)
        }
        Expr::Member { object, .. }
        | Expr::Unary { expr: object, .. }
        | Expr::Try { expr: object, .. }
        | Expr::StarProj {
            receiver: object, ..
        } => contains_struct_proj(object),
        Expr::Binary { left, right, .. } => {
            contains_struct_proj(left) || contains_struct_proj(right)
        }
        Expr::Lambda(l) => contains_struct_proj(&l.body),
        Expr::List { items, .. } => items.iter().any(contains_struct_proj),
        Expr::TypedObject { fields, .. } | Expr::AnonObject { fields, .. } => fields
            .iter()
            .any(|f| f.value.as_ref().is_some_and(contains_struct_proj)),
        _ => false,
    }
}

#[test]
fn accepts_colon_colon_static_paths() {
    let program = parse_program(
        r#"
            Database::rename_field(User::user_name, display_name)
            "#,
    )
    .unwrap();
    let Expr::Call { callee, args, .. } = program.result.as_ref().unwrap() else {
        panic!("{:?}", program.result);
    };
    match callee.as_ref() {
        Expr::Member {
            object,
            name,
            sep: PathSep::ColonColon,
            ..
        } => {
            assert!(matches!(object.as_ref(), Expr::Name { name, .. } if name == "Database"));
            assert_eq!(name, "rename_field");
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(args.len(), 2);
    match &args[0] {
        Expr::Member {
            object,
            name,
            sep: PathSep::ColonColon,
            ..
        } => {
            assert!(matches!(object.as_ref(), Expr::Name { name, .. } if name == "User"));
            assert_eq!(name, "user_name");
        }
        other => panic!("{other:?}"),
    }
    assert!(matches!(&args[1], Expr::Name { name, .. } if name == "display_name"));
}

#[test]
fn dot_and_colon_colon_share_path_shape() {
    let via_colon = parse_program("User::filter(x => true).collect()").unwrap();
    let via_dot = parse_program("User.filter(x => true).collect()").unwrap();
    fn callee_chain(expr: &Expr) -> Vec<(&str, Option<PathSep>)> {
        match expr {
            Expr::Call { callee, .. } => callee_chain(callee),
            Expr::Member {
                object, name, sep, ..
            } => {
                let mut v = callee_chain(object);
                v.push((name.as_str(), Some(*sep)));
                v
            }
            Expr::Name { name, .. } => vec![(name.as_str(), None)],
            _ => Vec::new(),
        }
    }
    let c = callee_chain(via_colon.result.as_ref().unwrap());
    let d = callee_chain(via_dot.result.as_ref().unwrap());
    assert_eq!(
        c.iter().map(|(n, _)| *n).collect::<Vec<_>>(),
        d.iter().map(|(n, _)| *n).collect::<Vec<_>>()
    );
    assert_eq!(c[1].1, Some(PathSep::ColonColon));
    assert_eq!(d[1].1, Some(PathSep::Dot));
}
