//! Conformance for the `vector<N>` contract: dimension resolution, precision,
//! finite components, serialization order and dimension diagnostics.

use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use serde_json::json;
use vos::{
    ContractDiagnostic, FieldIdentity, IDENTITY_MANIFEST_VERSION, IdentityManifest,
    ResolvedCanonicalType, ResolvedContract, TypeContractKind, TypeIdentity, VectorElement,
    VectorValue, VectorValueWire, parse_oak, resolve_contract,
};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../documentation/fixtures/contracts/vector")
}

fn read(relative: &str) -> String {
    fs::read_to_string(fixtures().join(relative))
        .unwrap_or_else(|error| panic!("{relative} does not read: {error}"))
}

fn manifest(path: &str, field: &str) -> IdentityManifest {
    IdentityManifest {
        format_version: IDENTITY_MANIFEST_VERSION.to_owned(),
        types: vec![TypeIdentity {
            canonical_path: vec![path.to_owned()],
            type_id: 1,
            kind: TypeContractKind::Class,
            fields: vec![FieldIdentity {
                canonical_name: field.to_owned(),
                field_id: 2,
                virtual_field_index: Some(0),
            }],
        }],
    }
}

fn resolve(
    relative: &str,
    path: &str,
    field: &str,
) -> Result<ResolvedContract, Vec<ContractDiagnostic>> {
    let projection = parse_oak(&read(relative))
        .expect("Oak parses the fixture")
        .project_schema()
        .expect("the fixture projects");
    resolve_contract(&projection, &manifest(path, field))
}

fn resolved_fixture() -> ResolvedContract {
    let projection = parse_oak(&read("schema.vos"))
        .expect("Oak parses the vector fixture")
        .project_schema()
        .expect("the vector fixture projects");
    let manifest = IdentityManifest::from_json(&read("schema.manifest.json"))
        .expect("the vector fixture manifest is strict");
    resolve_contract(&projection, &manifest).expect("the vector fixture resolves")
}

#[test]
fn vector_fields_resolve_to_a_fixed_dimension_and_precision() {
    let contract = resolved_fixture();
    contract
        .validate()
        .expect("the resolved vector contract validates");

    let embedding = contract
        .types
        .iter()
        .find(|item| item.canonical_path == ["demo", "embeddings", "Embedding"])
        .expect("the fixture table resolves");
    for (name, dimension) in [("coarse", 3_u32), ("fine", 8)] {
        let field = embedding
            .fields
            .iter()
            .find(|field| field.canonical_name == name)
            .unwrap_or_else(|| panic!("{name} survives resolution"));
        assert_eq!(
            field.canonical_type,
            ResolvedCanonicalType::Vector {
                element: VectorElement::F32,
                dimension,
            },
            "{name} must carry its authored dimension and the `f32` precision"
        );
        assert_eq!(
            serde_json::to_value(&field.canonical_type).unwrap(),
            json!({"vector": {"element": "f32", "dimension": dimension}})
        );
    }

    let decoded =
        ResolvedContract::from_json(&contract.to_json().unwrap()).expect("the artifact reads back");
    assert_eq!(decoded, contract);
}

#[test]
fn zero_dimension_is_diagnosed() {
    let diagnostics = resolve("zero_dimension.vos", "ZeroDimension", "value")
        .expect_err("a zero dimension must not resolve");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "RES020");
    assert_eq!(
        diagnostics[0].message,
        "vector dimension must be a positive decimal integer, found 0"
    );
}

#[test]
fn a_type_argument_is_not_a_dimension() {
    let diagnostics = resolve("type_argument.vos", "TypeArgument", "value")
        .expect_err("a type argument must not resolve");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "RES019");
    assert_eq!(
        diagnostics[0].message,
        "vector requires exactly one literal dimension"
    );
}

#[test]
fn a_bare_vector_is_diagnosed() {
    let diagnostics = resolve("no_dimension.vos", "NoDimension", "value")
        .expect_err("a bare `vector` must not resolve");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "RES019");
    assert_eq!(
        diagnostics[0].message,
        "vector requires a literal dimension"
    );
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Values {
    format_version: String,
    note: String,
    cases: Vec<ValueCase>,
    non_finite: Vec<NonFiniteCase>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ValueCase {
    id: String,
    wire: VectorValueWire,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct NonFiniteCase {
    id: String,
    component: String,
    expect_code: String,
}

fn non_finite(token: &str) -> f32 {
    match token {
        "nan" => f32::NAN,
        "positive-infinity" => f32::INFINITY,
        "negative-infinity" => f32::NEG_INFINITY,
        other => panic!("unknown non-finite token {other}"),
    }
}

#[test]
fn positive_value_fixtures_keep_dimension_precision_and_order() {
    let values: Values = serde_json::from_str(&read("values.json")).expect("values.json parses");
    assert_eq!(values.format_version, "vos-vector-value-fixture-v0");
    assert!(!values.note.trim().is_empty());
    assert!(!values.cases.is_empty());

    for case in &values.cases {
        let value = VectorValue::from_wire(case.wire.clone())
            .unwrap_or_else(|error| panic!("{} wire does not read: {}", case.id, error.code));
        assert_eq!(value.dimension(), case.wire.dimension, "{}", case.id);
        assert_eq!(value.element(), VectorElement::F32);
        assert_eq!(
            value.components(),
            case.wire.components.as_slice(),
            "{} components keep their serialized order",
            case.id
        );
        assert_eq!(value.to_wire(), case.wire, "{} is not canonical", case.id);
        assert_eq!(
            VectorValue::from_json(&value.to_json().unwrap()).unwrap(),
            value,
            "{} must survive the artifact round-trip",
            case.id
        );
    }
}

#[test]
fn non_finite_fixture_tokens_are_rejected() {
    let values: Values = serde_json::from_str(&read("values.json")).expect("values.json parses");
    assert!(!values.non_finite.is_empty());

    for case in &values.non_finite {
        let dimension = 3;
        let error = VectorValue::new(dimension, vec![0.0, non_finite(&case.component), 0.0])
            .expect_err("a non-finite component must not become a value");
        assert_eq!(error.code, case.expect_code, "{} code", case.id);
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct InvalidIndex {
    format_version: String,
    note: String,
    cases: Vec<InvalidCase>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct InvalidCase {
    id: String,
    source: String,
    expect_code: String,
}

#[test]
fn negative_value_fixtures_are_rejected_with_the_reviewed_code() {
    let index: InvalidIndex =
        serde_json::from_str(&read("invalid.json")).expect("invalid.json parses");
    assert_eq!(index.format_version, "vos-vector-value-invalid-v0");
    assert!(!index.note.trim().is_empty());
    assert!(!index.cases.is_empty());

    for case in &index.cases {
        let input = read(&case.source);
        let error = match VectorValue::from_json(&input) {
            Ok(value) => panic!(
                "{} was accepted as dimension {}",
                case.id,
                value.dimension()
            ),
            Err(error) => error,
        };
        assert_eq!(error.code, case.expect_code, "{} code", case.id);
        assert!(!error.message.is_empty(), "{} carries a message", case.id);
    }
}
