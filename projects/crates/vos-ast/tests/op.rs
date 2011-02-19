use vos_ast::expr::{BinaryOp, Expr, FieldInit, ProjItem};
use vos_ast::op::{Exec, QueryPlan, Stage, TableRef, lambda1};
use vos_ast::{Literal, Span};

#[test]
fn query_plan_filter_map_collect_shape() {
    let s = Span::new(0, 1);
    let table = TableRef {
        name: "User".into(),
        span: s,
    };
    let pred = Expr::Lambda(lambda1(
        "x",
        Expr::Binary {
            op: BinaryOp::Ne,
            left: Box::new(Expr::member(Expr::name("x", s), "user_name", s)),
            right: Box::new(Expr::Literal(Literal::String(String::new()))),
            span: s,
        },
        s,
    ));
    let proj = Expr::StructProj {
        receiver: Box::new(Expr::name("x", s)),
        items: vec![
            ProjItem::Field(FieldInit::shorthand("user_id", s)),
            ProjItem::Field(FieldInit::named("name", Expr::name("user_name", s), s)),
        ],
        span: s,
    };
    let plan = QueryPlan::all(table)
        .filter(pred, s)
        .map(Expr::Lambda(lambda1("x", proj, s)), s);
    assert_eq!(plan.stages.len(), 3);
    let exec = Exec::Collect { plan, span: s };
    match exec {
        Exec::Collect { plan, .. } => {
            assert_eq!(plan.source.name, "User");
            assert!(matches!(plan.stages[1], Stage::Filter { .. }));
            assert!(matches!(plan.stages[2], Stage::Map { .. }));
        }
        other => panic!("unexpected {other:?}"),
    }
}
