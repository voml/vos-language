use vos_ast::op::{QueryPlan, Stage, TableRef};
use vos_inspect::{InspectConfig, InspectEngine, InspectId, InspectLevel, codes};

fn schema() -> vos_ast::Document {
    vos_parser::parse_document(
        r#"
        table User {
            @@user_id: uuid,
            @user_name: utf8,
        }
        table Post {
            @@post_id: uuid,
            author: &User,
            title: utf8,
        }
        "#,
    )
    .expect("schema")
}

#[test]
fn prefer_explicit_load_silent_by_default() {
    let program = vos_parser::parse_program(
        r#"
        Post
            .map(x => x.{ title, author_name: author.user_name })
            .collect()
        "#,
    )
    .expect("program");
    let report =
        InspectEngine::new().inspect_program(&program, &schema(), &InspectConfig::permissive());
    assert!(report.findings.is_empty());
    assert!(report.passed());
}

#[test]
fn prefer_explicit_load_warns_when_configured() {
    let program = vos_parser::parse_program(
        r#"
        Post
            .map(x => x.{ title, author_name: author.user_name })
            .collect()
        "#,
    )
    .expect("program");
    let mut cfg = InspectConfig::permissive();
    cfg.set(InspectId::PreferExplicitLoad, InspectLevel::Warn);
    let report = InspectEngine::new().inspect_program(&program, &schema(), &cfg);
    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].id, InspectId::PreferExplicitLoad);
    assert_eq!(report.findings[0].level, InspectLevel::Warn);
    assert_eq!(report.findings[0].id.code(), codes::PREFER_EXPLICIT_LOAD);
    assert!(report.passed());
}

#[test]
fn prefer_explicit_load_silent_when_load_present() {
    let program = vos_parser::parse_program(
        r#"
        Post
            .load(x => x.author)
            .map(x => x.{ title, author_name: author.user_name })
            .collect()
        "#,
    )
    .expect("program");
    let mut cfg = InspectConfig::permissive();
    cfg.set(InspectId::PreferExplicitLoad, InspectLevel::Deny);
    let report = InspectEngine::new().inspect_program(&program, &schema(), &cfg);
    assert!(report.findings.is_empty());
    assert!(report.passed());
}

#[test]
fn prefer_explicit_load_deny_fails_report() {
    let mut plan = QueryPlan::all(TableRef {
        name: "Post".into(),
        span: vos_ast::Span::empty(0),
    });
    let map = vos_parser::parse_program(
        r#"Post.map(x => x.{ author_name: author.user_name }).collect()"#,
    )
    .unwrap();
    let mut cfg = InspectConfig::permissive();
    cfg.set(InspectId::PreferExplicitLoad, InspectLevel::Deny);
    let report = InspectEngine::new().inspect_program(&map, &schema(), &cfg);
    assert!(!report.passed());
    assert_eq!(report.denials().count(), 1);

    plan.stages.push(Stage::Map {
        projection: vos_ast::Expr::name("x", vos_ast::Span::empty(0)),
        span: vos_ast::Span::empty(0),
    });
    let clean = InspectEngine::new().inspect_plan(&plan, &schema(), &cfg);
    assert!(clean.passed());
}

#[test]
fn config_accepts_code_or_name_keys() {
    let mut cfg = InspectConfig::permissive();
    cfg.by_rule
        .insert(codes::PREFER_EXPLICIT_LOAD.to_owned(), InspectLevel::Warn);
    assert_eq!(cfg.level(InspectId::PreferExplicitLoad), InspectLevel::Warn);

    let mut cfg2 = InspectConfig::permissive();
    cfg2.by_rule
        .insert("prefer-explicit-load".into(), InspectLevel::Deny);
    assert_eq!(
        cfg2.level(InspectId::PreferExplicitLoad),
        InspectLevel::Deny
    );
}
