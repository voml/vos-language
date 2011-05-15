//! Operation programs are projected by Oak and consumed through the VOS facade.

use std::fs;
use std::path::{Path, PathBuf};

use vos::syntax::{VosExpr, VosStmt};

fn operations_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../specifications/fixtures/operations")
}

fn operation_sources(dir: &Path) -> Vec<PathBuf> {
    let mut files = fs::read_dir(dir)
        .expect("operation fixture directory")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension().and_then(|extension| extension.to_str()) == Some("vos")
                && !path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.ends_with(".normalized.vos"))
        })
        .collect::<Vec<_>>();
    files.sort();
    files
}

#[test]
fn operation_fixtures_parse_into_programs() {
    let files = operation_sources(&operations_root());
    assert!(!files.is_empty(), "operation fixture set is empty");

    for path in files {
        let source = fs::read_to_string(&path).expect("fixture source");
        let program = vos::parse_program(&source)
            .unwrap_or_else(|error| panic!("Oak rejected {}: {error}", path.display()));
        assert!(
            program.result.is_some(),
            "{} must end with a result expression",
            path.display()
        );
        assert_eq!(&source[program.span], source.trim());
    }
}

#[test]
fn lazy_pipeline_program_exposes_statements_and_result() {
    let source = fs::read_to_string(operations_root().join("filter_map_collect.vos"))
        .expect("fixture source");
    let program = vos::parse_program(&source).expect("Oak projects the pipeline");

    assert!(program.micros.is_empty());
    assert_eq!(program.statements.len(), 3);
    assert!(matches!(
        &program.statements[1],
        VosStmt::Let(statement) if statement.name == "public_users"
    ));
    assert!(matches!(
        program.result.as_ref(),
        Some(VosExpr::Name { name, .. }) if name == "users"
    ));
}

#[test]
fn operation_declarations_carry_their_body_programs() {
    let input = vos::parse_oak(
        "query active(status: utf8) -> [User] { User.filter(x => x.active).collect() }",
    )
    .expect("Oak parses VOS");

    let program = input.declarations()[0]
        .body_program
        .as_ref()
        .expect("operation body projects into a program");
    let Some(VosExpr::Call { callee, args, .. }) = program.result.as_ref() else {
        panic!("expected a trailing call, found {:?}", program.result);
    };
    assert!(args.is_empty());
    assert!(matches!(
        callee.as_ref(),
        VosExpr::Member { name, .. } if name == "collect"
    ));
}

#[test]
fn facade_rejects_incomplete_programs() {
    assert!(vos::parse_program("let value = ").is_err());
    assert!(vos::parse_program("User.filter(").is_err());
    assert!(vos::parse_program("let = User").is_err());
}
