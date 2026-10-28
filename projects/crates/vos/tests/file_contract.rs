//! Conformance for the `file` contract: declaration, resolution, descriptor
//! identity, read windows and diagnostics over the reviewed `file` fixtures.

use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use serde_json::json;
use vos::{
    BytesValue, FieldIdentity, FileDescriptor, FileManifestIdentity, FileMetadata, FileReadWindow,
    FileReference, IDENTITY_MANIFEST_VERSION, IdentityManifest, ResolvedCanonicalType,
    ResolvedContract, TypeContractKind, TypeIdentity, parse_oak, resolve_contract,
};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../documentation/fixtures/contracts/file")
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

#[test]
fn file_fields_resolve_to_the_descriptor_type_and_serialize_as_one_spelling() {
    let projection = parse_oak(&read("schema.vos"))
        .expect("Oak parses the file fixture")
        .project_schema()
        .expect("the file fixture projects");
    let manifest = IdentityManifest::from_json(&read("schema.manifest.json"))
        .expect("the file fixture manifest is strict");
    let contract = resolve_contract(&projection, &manifest).expect("the file fixture resolves");
    contract
        .validate()
        .expect("the resolved file contract validates");

    let attachment = contract
        .types
        .iter()
        .find(|item| item.canonical_path == ["demo", "files", "Attachment"])
        .expect("the fixture table resolves");
    for name in ["handle", "preview"] {
        let field = attachment
            .fields
            .iter()
            .find(|field| field.canonical_name == name)
            .unwrap_or_else(|| panic!("{name} survives resolution"));
        assert_eq!(
            field.canonical_type,
            ResolvedCanonicalType::File,
            "{name} must resolve to the logical `file` type"
        );
        assert_eq!(
            serde_json::to_value(&field.canonical_type).unwrap(),
            json!("file")
        );
    }

    let decoded =
        ResolvedContract::from_json(&contract.to_json().unwrap()).expect("the artifact reads back");
    assert_eq!(decoded, contract);
}

#[test]
fn file_with_type_arguments_is_diagnosed_instead_of_sealed_as_a_generic() {
    let projection = parse_oak(&read("generic_arguments.vos"))
        .expect("Oak parses the negative fixture")
        .project_schema()
        .expect("the negative fixture projects");
    let manifest = manifest("BadFile", "handle");

    let diagnostics = resolve_contract(&projection, &manifest)
        .expect_err("`file` must not accept type arguments");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "RES018");
    assert_eq!(
        diagnostics[0].message,
        "file is a logical type and does not take type arguments"
    );
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DescriptorFixture {
    format_version: String,
    note: String,
    content: vos::BytesValueWire,
    descriptor: FileDescriptor,
    expect: Expect,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Expect {
    full_window: FileReadWindow,
    reads: Vec<ReadCase>,
    released_code: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReadCase {
    request: FileReadWindow,
    window: FileReadWindow,
    eof: bool,
}

#[test]
fn descriptor_fixture_recomputes_its_identity_and_honours_its_windows() {
    let fixture: DescriptorFixture =
        serde_json::from_str(&read("descriptor.json")).expect("descriptor.json parses");
    assert_eq!(fixture.format_version, "vos-file-descriptor-fixture-v0");
    assert!(!fixture.note.trim().is_empty());

    let content = BytesValue::from_wire(fixture.content)
        .expect("the fixture content is a strict bytes value");
    let descriptor = FileDescriptor::from_json(
        &serde_json::to_string(&fixture.descriptor).expect("the fixture descriptor serializes"),
    )
    .expect("the fixture descriptor is strict");

    assert_eq!(
        descriptor.identity,
        FileManifestIdentity::from_octets(content.as_octets()),
        "the reviewed digest and length must reproduce from the content octets"
    );
    assert_eq!(
        descriptor,
        FileDescriptor::describe(
            content.as_octets(),
            FileMetadata {
                name: descriptor.metadata.name.clone(),
                media_type: descriptor.metadata.media_type.clone(),
            }
        )
    );
    assert_eq!(descriptor.full_window(), fixture.expect.full_window);

    let mut reference = FileReference::live(descriptor.clone());
    for case in &fixture.expect.reads {
        assert_eq!(
            reference
                .load(case.request)
                .expect("a live reference reads"),
            case.window,
            "a {:?} request must resolve to the reviewed window",
            case.request
        );
        assert_eq!(descriptor.is_eof(case.request.offset), case.eof);
    }

    reference.release();
    assert_eq!(
        reference
            .load(descriptor.full_window())
            .expect_err("a released reference must not read")
            .code,
        fixture.expect.released_code
    );
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
fn negative_descriptor_fixtures_are_rejected_with_the_reviewed_code() {
    let index: InvalidIndex =
        serde_json::from_str(&read("invalid.json")).expect("invalid.json parses");
    assert_eq!(index.format_version, "vos-file-descriptor-invalid-v0");
    assert!(!index.note.trim().is_empty());
    assert!(!index.cases.is_empty());

    for case in &index.cases {
        let input = read(&case.source);
        let error = match FileDescriptor::from_json(&input) {
            Ok(descriptor) => panic!("{} was accepted as {} octets", case.id, descriptor.octets()),
            Err(error) => error,
        };
        assert_eq!(error.code, case.expect_code, "{} code", case.id);
        assert!(!error.message.is_empty(), "{} carries a message", case.id);
    }
}
