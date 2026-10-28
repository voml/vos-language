//! Conformance for the `bytes` contract: declaration, resolution, value wire
//! and diagnostics over the reviewed `bytes` fixtures.

use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use serde_json::json;
use vos::{
    BytesPlacement, BytesPlacementPolicy, BytesValue, BytesValueWire, CanonicalScalar,
    FieldIdentity, IDENTITY_MANIFEST_VERSION, IdentityManifest, ResolvedCanonicalType,
    ResolvedContract, TypeContractKind, TypeIdentity, parse_oak, resolve_contract,
};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../documentation/fixtures/contracts/bytes")
}

fn read(relative: &str) -> String {
    fs::read_to_string(fixtures().join(relative))
        .unwrap_or_else(|error| panic!("{relative} does not read: {error}"))
}

fn resolved_fixture() -> ResolvedContract {
    let projection = parse_oak(&read("schema.vos"))
        .expect("Oak parses the bytes fixture")
        .project_schema()
        .expect("the bytes fixture projects");
    let manifest = IdentityManifest::from_json(&read("schema.manifest.json"))
        .expect("the bytes fixture manifest is strict");
    resolve_contract(&projection, &manifest).expect("the bytes fixture resolves")
}

#[test]
fn bytes_fields_resolve_to_the_shared_scalar_and_serialize_as_one_spelling() {
    let contract = resolved_fixture();
    contract
        .validate()
        .expect("the resolved bytes contract validates");

    let blob = contract
        .types
        .iter()
        .find(|item| item.canonical_path == ["demo", "binaries", "Blob"])
        .expect("the fixture table resolves");
    for name in ["payload", "digest"] {
        let field = blob
            .fields
            .iter()
            .find(|field| field.canonical_name == name)
            .unwrap_or_else(|| panic!("{name} survives resolution"));
        assert_eq!(
            field.canonical_type,
            ResolvedCanonicalType::Scalar(CanonicalScalar::Bytes),
            "{name} must resolve to the shared `bytes` scalar, not to an opaque generic"
        );
        assert_eq!(
            serde_json::to_value(&field.canonical_type).unwrap(),
            json!({"scalar": "bytes"})
        );
    }

    let decoded =
        ResolvedContract::from_json(&contract.to_json().unwrap()).expect("the artifact reads back");
    assert_eq!(decoded, contract);
}

#[test]
fn bytes_with_type_arguments_is_diagnosed_instead_of_sealed_as_a_generic() {
    let projection = parse_oak("class Bad { payload: bytes<utf8> }")
        .expect("Oak parses the negative fixture")
        .project_schema()
        .expect("the negative fixture projects");
    let manifest = IdentityManifest {
        format_version: IDENTITY_MANIFEST_VERSION.to_owned(),
        types: vec![TypeIdentity {
            canonical_path: vec!["Bad".to_owned()],
            type_id: 1,
            kind: TypeContractKind::Class,
            fields: vec![FieldIdentity {
                canonical_name: "payload".to_owned(),
                field_id: 2,
                virtual_field_index: Some(0),
            }],
        }],
    };

    let diagnostics = resolve_contract(&projection, &manifest)
        .expect_err("`bytes` is a scalar and must not accept arguments");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "RES017");
    assert_eq!(
        diagnostics[0].message,
        "bytes is a scalar and does not take type arguments"
    );
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Values {
    format_version: String,
    note: String,
    host_placement: HostPlacement,
    cases: Vec<ValueCase>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct HostPlacement {
    kind: String,
    inline_limit_octets: usize,
    note: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ValueCase {
    id: String,
    octets: usize,
    wire: BytesValueWire,
    placement: BytesPlacement,
}

struct FixtureHost {
    inline_limit_octets: usize,
}

impl BytesPlacementPolicy for FixtureHost {
    fn placement_for(&self, octets: &[u8]) -> BytesPlacement {
        if octets.len() <= self.inline_limit_octets {
            BytesPlacement::Inline
        } else {
            BytesPlacement::ObjectReference
        }
    }
}

#[test]
fn positive_value_fixtures_round_trip_and_follow_the_host_declaration() {
    let values: Values = serde_json::from_str(&read("values.json")).expect("values.json parses");
    assert_eq!(values.format_version, "vos-bytes-value-fixture-v0");
    assert!(!values.note.trim().is_empty());
    assert!(!values.host_placement.kind.trim().is_empty());
    assert!(!values.host_placement.note.trim().is_empty());
    assert!(!values.cases.is_empty());

    let host = FixtureHost {
        inline_limit_octets: values.host_placement.inline_limit_octets,
    };
    for case in &values.cases {
        let value = BytesValue::from_wire(case.wire.clone())
            .unwrap_or_else(|error| panic!("{} wire does not read: {}", case.id, error.code));
        assert_eq!(value.len(), case.octets, "{} octet count", case.id);
        assert_eq!(value.to_wire(), case.wire, "{} is not canonical", case.id);
        assert_eq!(
            value.placement(&host),
            case.placement,
            "{} placement follows the host declaration",
            case.id
        );
        assert_eq!(
            BytesValue::from_json(&value.to_json().unwrap()).unwrap(),
            value,
            "{} must survive the artifact round-trip",
            case.id
        );
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
    assert_eq!(index.format_version, "vos-bytes-value-invalid-v0");
    assert!(!index.note.trim().is_empty());
    assert!(!index.cases.is_empty());

    for case in &index.cases {
        let input = read(&case.source);
        let error = match BytesValue::from_json(&input) {
            Ok(value) => panic!("{} was accepted as {} octets", case.id, value.len()),
            Err(error) => error,
        };
        assert_eq!(error.code, case.expect_code, "{} code", case.id);
        assert!(!error.message.is_empty(), "{} carries a message", case.id);
    }
}
