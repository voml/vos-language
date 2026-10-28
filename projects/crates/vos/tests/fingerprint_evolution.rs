//! Fingerprint evolution matrix for the schema and artifact contract families.
//!
//! Every case is a real `.vos` source plus its identity manifest, so the matrix
//! fixes what a reorder, rename, retype or re-declaration may change. Two
//! fingerprints are observed per case:
//!
//! * `schemaFingerprint` — the resolved schema contract fingerprint
//!   (`resolve_contract`), covering types, fields, canonical types, attributes
//!   and defaults.
//! * `artifactFingerprint` — the sealed product artifact fingerprint
//!   (`ContractArtifact`), covering operations, effect, capabilities and
//!   services.
//!
//! The verdicts are the compatibility decision a downstream host makes: `same`
//! means a previously sealed fingerprint still validates against the new
//! content, `changed` means it must be rejected (`RES014` for the schema,
//! `ART020` for the artifact).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use vos::{
    ContractArtifact, ContractFormat, ContractPayload, ContractProducer, IDENTITY_MANIFEST_VERSION,
    IdentityManifest, MethodStreamMode, OperationContract, RESOLVED_CONTRACT_FORMAT_VERSION,
    ResolvedContract, ServiceContractDecl, ServiceMethodDecl, build_operation_contracts,
    build_service_contracts, parse_oak, parse_operation_semantics_from_attributes, resolve_contract,
};

const FORMAT_VERSION: &str = "vos-fingerprint-evolution-v0";

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Matrix {
    format_version: String,
    base: String,
    cases: Vec<Case>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Case {
    id: String,
    source: String,
    manifest: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    bindings: BTreeMap<String, String>,
    expect: Expect,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Expect {
    schema_fingerprint: String,
    artifact_fingerprint: String,
    schema_verdict: String,
    artifact_verdict: String,
}

/// The two fingerprints a case produces.
struct Outcome {
    schema_fingerprint: String,
    types: Vec<vos::ResolvedTypeContract>,
    artifact: ContractArtifact,
}

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../documentation/fixtures/contracts/fingerprint_evolution")
}

fn matrix_path() -> PathBuf {
    fixtures().join("matrix.json")
}

fn load_matrix() -> Matrix {
    let matrix: Matrix = serde_json::from_str(
        &fs::read_to_string(matrix_path()).expect("fingerprint evolution matrix exists"),
    )
    .expect("matrix.json is a strict fingerprint evolution matrix");
    assert_eq!(
        matrix.format_version, FORMAT_VERSION,
        "matrix must declare its own format version"
    );
    matrix
}

fn build(case: &Case) -> Outcome {
    let directory = fixtures();
    let source = fs::read_to_string(directory.join(&case.source))
        .unwrap_or_else(|error| panic!("{}: {error}", case.source));
    let manifest_json = fs::read_to_string(directory.join(&case.manifest))
        .unwrap_or_else(|error| panic!("{}: {error}", case.manifest));
    let manifest = IdentityManifest::from_json(&manifest_json)
        .unwrap_or_else(|error| panic!("{}: {}", case.manifest, error.message));

    let input = parse_oak(&source).unwrap_or_else(|error| panic!("{}: {error}", case.id));
    let projection = input
        .project_schema()
        .unwrap_or_else(|diagnostics| panic!("{}: {diagnostics:?}", case.id));
    let contract = resolve_contract(&projection, &manifest)
        .unwrap_or_else(|diagnostics| panic!("{}: {diagnostics:?}", case.id));

    let operations = input
        .project_operations()
        .unwrap_or_else(|diagnostics| panic!("{}: {diagnostics:?}", case.id));
    let semantics = operations
        .iter()
        .map(|operation| {
            parse_operation_semantics_from_attributes(operation)
                .unwrap_or_else(|diagnostics| panic!("{}: {diagnostics:?}", case.id))
        })
        .collect::<Vec<_>>();
    let operation_contracts = build_operation_contracts(
        &[(0, &operations)],
        &semantics,
        &contract.types,
        None,
    )
    .unwrap_or_else(|diagnostics| panic!("{}: {diagnostics:?}", case.id));

    let services = input
        .project_services()
        .unwrap_or_else(|diagnostics| panic!("{}: {diagnostics:?}", case.id));
    let by_name = operation_contracts
        .iter()
        .map(|operation| (operation.name.as_str(), operation))
        .collect::<BTreeMap<_, _>>();
    let declarations = services
        .iter()
        .map(|service| ServiceContractDecl {
            service_id: service.service_id.clone(),
            methods: service
                .methods
                .iter()
                .map(|method| ServiceMethodDecl {
                    name: method.name.clone(),
                    request_type: method.parameters[0].type_expr.clone(),
                    response_type: method
                        .return_type
                        .clone()
                        .unwrap_or_else(|| panic!("{}: service method has no return type", case.id)),
                    stream: if method.stream {
                        MethodStreamMode::ServerStream
                    } else {
                        MethodStreamMode::Unary
                    },
                    operation_id: bound_operation(&by_name, method.name.as_str(), case)
                        .operation_id
                        .clone(),
                })
                .collect(),
        })
        .collect::<Vec<_>>();
    let service_contracts =
        build_service_contracts(&[(0, &services)], &declarations, &operation_contracts, &contract.types, None)
            .unwrap_or_else(|diagnostics| panic!("{}: {diagnostics:?}", case.id));

    let artifact = ContractArtifact::seal(
        ContractFormat::Iris,
        ContractProducer::new("fixture", "0"),
        ContractPayload {
            operations: operation_contracts,
            services: service_contracts,
        },
    );
    artifact
        .validate()
        .unwrap_or_else(|error| panic!("{}: {} {}", case.id, error.code, error.message));

    Outcome {
        schema_fingerprint: contract.schema_fingerprint,
        types: contract.types,
        artifact,
    }
}

fn bound_operation<'a>(
    by_name: &'a BTreeMap<&'a str, &'a OperationContract>,
    method: &str,
    case: &Case,
) -> &'a OperationContract {
    let name = case.bindings.get(method).map(String::as_str).unwrap_or(method);
    by_name.get(name).copied().unwrap_or_else(|| {
        panic!(
            "{}: service method {method} binds unknown operation {name}",
            case.id
        )
    })
}

fn assert_is_fingerprint(value: &str, label: &str) {
    assert_eq!(value.len(), 64, "{label} must be a 64 hex digest");
    assert!(
        value.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "{label} must be hexadecimal"
    );
}

#[test]
fn fingerprint_evolution_matrix_matches_its_recorded_verdicts() {
    let mut matrix = load_matrix();
    let update = std::env::var_os("VOS_UPDATE_FIXTURES").is_some();
    let base_id = matrix.base.clone();
    let base = build(
        matrix
            .cases
            .iter()
            .find(|case| case.id == base_id)
            .unwrap_or_else(|| panic!("matrix base case {base_id} is missing")),
    );
    assert_is_fingerprint(&base.schema_fingerprint, "base schema fingerprint");
    assert_is_fingerprint(&base.artifact.fingerprint, "base artifact fingerprint");

    let mut changed = false;
    for case in &mut matrix.cases {
        let outcome = build(case);
        if update {
            case.expect.schema_fingerprint = outcome.schema_fingerprint.clone();
            case.expect.artifact_fingerprint = outcome.artifact.fingerprint.clone();
            changed = true;
        }
        assert_is_fingerprint(&outcome.schema_fingerprint, &case.id);
        assert_eq!(
            case.expect.schema_fingerprint, outcome.schema_fingerprint,
            "{}: schema fingerprint drifted. Run VOS_UPDATE_FIXTURES=1 to review and re-record it",
            case.id
        );
        assert_eq!(
            case.expect.artifact_fingerprint, outcome.artifact.fingerprint,
            "{}: artifact fingerprint drifted. Run VOS_UPDATE_FIXTURES=1 to review and re-record it",
            case.id
        );

        // A host that still holds the base fingerprint must either accept the
        // new content (same) or reject it with the family-specific code.
        let stale_contract = ResolvedContract {
            format_version: RESOLVED_CONTRACT_FORMAT_VERSION.to_owned(),
            identity_manifest_version: IDENTITY_MANIFEST_VERSION.to_owned(),
            schema_fingerprint: base.schema_fingerprint.clone(),
            types: outcome.types.clone(),
        };
        match case.expect.schema_verdict.as_str() {
            "same" => {
                assert_eq!(
                    outcome.schema_fingerprint, base.schema_fingerprint,
                    "{}: schemaVerdict says same but the fingerprint moved",
                    case.id
                );
                stale_contract
                    .validate()
                    .unwrap_or_else(|error| panic!("{}: {} {}", case.id, error.code, error.message));
            }
            "changed" => {
                assert_ne!(
                    outcome.schema_fingerprint, base.schema_fingerprint,
                    "{}: schemaVerdict says changed but the fingerprint did not move",
                    case.id
                );
                assert_eq!(
                    stale_contract.validate().unwrap_err().code,
                    "RES014",
                    "{}: a stale schema fingerprint must be rejected by RES014",
                    case.id
                );
            }
            other => panic!("{}: unknown schema verdict {other}", case.id),
        }

        let mut stale_artifact = outcome.artifact.clone();
        stale_artifact.producer = base.artifact.producer.clone();
        stale_artifact.fingerprint = base.artifact.fingerprint.clone();
        match case.expect.artifact_verdict.as_str() {
            "same" => {
                assert_eq!(
                    outcome.artifact.fingerprint, base.artifact.fingerprint,
                    "{}: artifactVerdict says same but the fingerprint moved",
                    case.id
                );
                stale_artifact
                    .validate()
                    .unwrap_or_else(|error| panic!("{}: {} {}", case.id, error.code, error.message));
            }
            "changed" => {
                assert_ne!(
                    outcome.artifact.fingerprint, base.artifact.fingerprint,
                    "{}: artifactVerdict says changed but the fingerprint did not move",
                    case.id
                );
                assert_eq!(
                    stale_artifact.validate().unwrap_err().code,
                    "ART020",
                    "{}: a stale artifact fingerprint must be rejected by ART020",
                    case.id
                );
            }
            other => panic!("{}: unknown artifact verdict {other}", case.id),
        }
    }

    if changed {
        let serialized = serde_json::to_string_pretty(&matrix).expect("matrix serializes");
        fs::write(matrix_path(), format!("{serialized}\n")).expect("matrix is writable");
    }
}

#[test]
fn matrix_covers_every_schema_and_artifact_verdict_combination() {
    let matrix = load_matrix();
    let combinations = matrix
        .cases
        .iter()
        .map(|case| {
            (
                case.expect.schema_verdict.clone(),
                case.expect.artifact_verdict.clone(),
            )
        })
        .collect::<BTreeSet<_>>();
    for expected in [
        ("same", "same"),
        ("changed", "same"),
        ("same", "changed"),
        ("changed", "changed"),
    ] {
        assert!(
            combinations.contains(&(expected.0.to_owned(), expected.1.to_owned())),
            "the matrix must cover schema {} and artifact {}",
            expected.0,
            expected.1
        );
    }
}

#[test]
fn matrix_case_ids_are_unique_and_sources_exist() {
    let matrix = load_matrix();
    assert!(!matrix.cases.is_empty(), "the matrix must carry cases");
    let directory = fixtures();
    let mut ids = BTreeSet::new();
    for case in &matrix.cases {
        assert!(ids.insert(case.id.as_str()), "duplicate case id {}", case.id);
        assert!(
            directory.join(&case.source).is_file(),
            "{}: missing source {}",
            case.id,
            case.source
        );
        assert!(
            directory.join(&case.manifest).is_file(),
            "{}: missing manifest {}",
            case.id,
            case.manifest
        );
        for (method, bound) in &case.bindings {
            assert!(!method.is_empty(), "{}: empty method binding key", case.id);
            assert!(!bound.is_empty(), "{}: empty operation binding", case.id);
        }
    }
    assert!(
        ids.contains(matrix.base.as_str()),
        "matrix base {} must be one of the cases",
        matrix.base
    );
}
