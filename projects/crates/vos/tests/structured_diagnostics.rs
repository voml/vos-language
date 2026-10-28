//! Contract-level diagnostic golden: stable code, message and source span.
//!
//! Every case is a real `.vos` fixture under
//! `documentation/fixtures/contracts/diagnostics`. The golden records the exact
//! `{code, message, span}` triple a host can assert on, so a diagnostic that
//! silently loses its location is a test failure instead of a cosmetic change.
//!
//! `cargo test -p vos --offline --test structured_diagnostics` compares against
//! the reviewed golden. Set `VOS_UPDATE_FIXTURES=1` to rewrite it.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use vos::{
    CanonicalType, ContractDiagnostic, MethodStreamMode, OperationId, ServiceContractDecl,
    ServiceMethodDecl, build_operation_contracts, build_service_contracts, parse_oak,
    parse_operation_semantics_from_attributes,
};

const FORMAT_VERSION: &str = "vos-diagnostic-golden-v0";

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Golden {
    format_version: String,
    cases: Vec<Case>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Case {
    id: String,
    kind: String,
    source: String,
    expect: Vec<Expect>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Expect {
    code: String,
    message: String,
    span: Option<Span>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Span {
    start: usize,
    end: usize,
}

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../documentation/fixtures/contracts/diagnostics")
}

fn golden_path() -> PathBuf {
    fixtures().join("codes.json")
}

fn read(path: &PathBuf) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn observed(diagnostics: &[ContractDiagnostic]) -> Vec<Expect> {
    diagnostics
        .iter()
        .map(|diagnostic| Expect {
            code: diagnostic.code.clone(),
            message: diagnostic.message.clone(),
            span: diagnostic.span.as_ref().map(|span| Span {
                start: span.start,
                end: span.end,
            }),
        })
        .collect()
}

/// Drives one case from its fixture source to the diagnostics it produces.
fn run(case: &Case) -> Vec<ContractDiagnostic> {
    let source = read(&fixtures().join(&case.source));
    match case.kind.as_str() {
        "operation-semantics" => {
            let input = parse_oak(&source).expect("Oak parses the fixture");
            let operations = input
                .project_operations()
                .expect("the fixture projects its operations");
            operations
                .iter()
                .flat_map(|operation| {
                    parse_operation_semantics_from_attributes(operation)
                        .err()
                        .unwrap_or_default()
                })
                .collect()
        }
        "operation-contracts" => {
            let input = parse_oak(&source).expect("Oak parses the fixture");
            let operations = input
                .project_operations()
                .expect("the fixture projects its operations");
            let semantics = operations
                .iter()
                .filter_map(|operation| {
                    parse_operation_semantics_from_attributes(operation).ok()
                })
                .collect::<Vec<_>>();
            build_operation_contracts(&[(0, &operations)], &semantics, &[], None)
                .expect_err("the fixture must fail operation contract construction")
        }
        "service-unknown-binding" | "service-request-mismatch" => {
            let input = parse_oak(&source).expect("Oak parses the fixture");
            let operations = input
                .project_operations()
                .expect("the fixture projects its operations");
            let semantics = operations
                .iter()
                .filter_map(|operation| {
                    parse_operation_semantics_from_attributes(operation).ok()
                })
                .collect::<Vec<_>>();
            let built = build_operation_contracts(&[(0, &operations)], &semantics, &[], None)
                .expect("the fixture operations build");
            let services = input
                .project_services()
                .expect("the fixture projects its service");
            let fetch = built
                .iter()
                .find(|operation| operation.name == "fetch")
                .expect("the fixture declares a fetch operation");
            let (request_type, operation_id) = if case.kind == "service-unknown-binding" {
                (
                    CanonicalType::Named(vec!["uuid".to_owned()]),
                    OperationId::parse(&format!("op_{}", "0".repeat(64)))
                        .expect("a fabricated operation id parses"),
                )
            } else {
                (
                    CanonicalType::Named(vec!["utf8".to_owned()]),
                    fetch.operation_id.clone(),
                )
            };
            let declaration = ServiceContractDecl {
                service_id: services[0].service_id.clone(),
                methods: vec![ServiceMethodDecl {
                    name: "fetch".to_owned(),
                    request_type,
                    response_type: CanonicalType::Named(vec!["uuid".to_owned()]),
                    stream: MethodStreamMode::Unary,
                    operation_id,
                }],
            };
            build_service_contracts(&[(0, &services)], &[declaration], &built, &[], None)
                .expect_err("the fixture must fail service contract construction")
        }
        other => panic!("unknown diagnostic case kind {other}"),
    }
}

fn load_golden() -> Golden {
    serde_json::from_str(&read(&golden_path())).expect("the diagnostic golden parses")
}

#[test]
fn diagnostic_golden_matches_every_case() {
    let golden = load_golden();
    assert_eq!(golden.format_version, FORMAT_VERSION);
    assert!(!golden.cases.is_empty(), "the diagnostic golden is empty");

    let mut actual = Vec::new();
    for case in &golden.cases {
        actual.push(Case {
            id: case.id.clone(),
            kind: case.kind.clone(),
            source: case.source.clone(),
            expect: observed(&run(case)),
        });
    }

    if std::env::var("VOS_UPDATE_FIXTURES").is_ok() {
        let rewritten = serde_json::to_string_pretty(&Golden {
            format_version: FORMAT_VERSION.to_owned(),
            cases: actual,
        })
        .expect("the diagnostic golden serializes");
        fs::write(golden_path(), format!("{rewritten}\n")).expect("the diagnostic golden writes");
        return;
    }

    for (expected, produced) in golden.cases.iter().zip(actual.iter()) {
        assert_eq!(produced.expect, expected.expect, "case {}", expected.id);
    }
    assert_eq!(actual.len(), golden.cases.len());
}

#[test]
fn golden_covers_operation_and_service_diagnostic_families() {
    let golden = load_golden();
    let codes = golden
        .cases
        .iter()
        .flat_map(|case| case.expect.iter())
        .map(|expect| expect.code.as_str())
        .collect::<Vec<_>>();

    for code in ["OPC001", "RES001", "SVC004", "SVC021"] {
        assert!(codes.contains(&code), "the golden must cover {code}");
    }
    assert!(
        golden
            .cases
            .iter()
            .flat_map(|case| case.expect.iter())
            .all(|expect| expect.span.is_some()),
        "every reviewed diagnostic must carry a source span"
    );
}
