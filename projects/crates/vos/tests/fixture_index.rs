//! Conformance fixture index: the machine-readable catalog a downstream host
//! (YYDB, YYDS catalog, Iris) reads instead of scanning the fixture tree.
//!
//! The index is the stable shape: a host looks a case up by `id`, opens `entry`
//! and follows `golden`, `manifest` and the nested sub-indexes. These tests keep
//! that shape honest by recomputing every golden the index points at and by
//! refusing to let a fixture file exist without being catalogued.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;
use vos::{ContractArtifact, IdentityManifest, parse_oak, resolve_contract};

const ROOT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../documentation/fixtures/contracts"
);

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Index {
    format_version: String,
    note: String,
    stages: Vec<String>,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Case {
    id: String,
    kind: String,
    stage: String,
    entry: String,
    files: Vec<String>,
    #[serde(default)]
    golden: Option<String>,
    #[serde(default)]
    manifest: Option<String>,
    #[serde(default)]
    schema_fingerprint: Option<String>,
    #[serde(default)]
    operations: Option<usize>,
    note: String,
}

fn root() -> PathBuf {
    PathBuf::from(ROOT)
}

fn read(relative: &str) -> String {
    std::fs::read_to_string(root().join(relative))
        .unwrap_or_else(|error| panic!("{relative} does not read: {error}"))
}

fn index() -> Index {
    serde_json::from_str(&read("index.json")).expect("fixture index parses")
}

fn cases_of<'a>(index: &'a Index, kind: &str) -> Vec<&'a Case> {
    index
        .cases
        .iter()
        .filter(|case| case.kind == kind)
        .collect()
}

fn collect_files(directory: &Path, base: &Path, files: &mut BTreeSet<String>) {
    for entry in std::fs::read_dir(directory).expect("fixture directory reads") {
        let entry = entry.expect("fixture entry reads");
        let path = entry.path();
        if entry.file_type().expect("fixture entry type").is_dir() {
            collect_files(&path, base, files);
        } else {
            let relative = path.strip_prefix(base).expect("fixture path is under its root");
            files.insert(
                relative
                    .components()
                    .map(|component| component.as_os_str().to_str().unwrap())
                    .collect::<Vec<_>>()
                    .join("/"),
            );
        }
    }
}

#[test]
fn index_catalogues_every_fixture_file_and_nothing_else() {
    let index = index();
    assert_eq!(index.format_version, "vos-contract-fixture-index-v0");
    assert!(!index.note.trim().is_empty());

    let mut ids = BTreeSet::new();
    for case in &index.cases {
        assert!(ids.insert(case.id.as_str()), "duplicate case id {}", case.id);
        assert!(
            index.stages.iter().any(|stage| stage == &case.stage),
            "{} uses unknown stage {}",
            case.id,
            case.stage
        );
        assert!(!case.note.trim().is_empty(), "{} has no note", case.id);
        assert!(
            case.files.iter().any(|file| file == &case.entry),
            "{} does not list its own entry",
            case.id
        );
        for path in case
            .files
            .iter()
            .chain(case.golden.iter())
            .chain(case.manifest.iter())
        {
            assert!(
                !path.contains('\\') && !path.starts_with('/') && !path.contains(".."),
                "{path} is not a relative forward-slash path"
            );
            assert!(
                root().join(path).is_file(),
                "{path} listed by {} does not exist",
                case.id
            );
        }
    }

    let mut declared = BTreeSet::new();
    for case in &index.cases {
        declared.extend(case.files.iter().cloned());
    }
    let mut on_disk = BTreeSet::new();
    collect_files(&root(), &root(), &mut on_disk);
    on_disk.remove("index.json");
    assert_eq!(
        declared, on_disk,
        "the index and the fixture tree disagree about what exists"
    );
}

#[test]
fn indexed_projection_and_resolution_goldens_still_reproduce() {
    let index = index();
    for case in cases_of(&index, "schema-projection") {
        let golden = read(case.golden.as_deref().expect("a projection case names its golden"));
        let projection = parse_oak(&read(&case.entry))
            .expect("Oak parses the indexed source")
            .project_schema()
            .expect("the indexed source projects");
        let actual: Value = serde_json::from_str(&projection.to_json().unwrap()).unwrap();
        let expected: Value = serde_json::from_str(&golden).unwrap();
        assert_eq!(actual, expected, "{} golden drifted", case.id);
    }
    let resolved = cases_of(&index, "resolved-schema");
    assert!(!resolved.is_empty(), "no resolved case is indexed");
    for case in resolved {
        let manifest = IdentityManifest::from_json(&read(
            case.manifest.as_deref().expect("a resolved case names its manifest"),
        ))
        .expect("the indexed identity manifest reads");
        let golden = read(case.golden.as_deref().expect("a resolved case names its golden"));
        let projection = parse_oak(&read(&case.entry))
            .expect("Oak parses the indexed source")
            .project_schema()
            .expect("the indexed source projects");
        let contract =
            resolve_contract(&projection, &manifest).expect("the indexed source resolves");
        let actual: Value = serde_json::from_str(&contract.to_json().unwrap()).unwrap();
        let expected: Value = serde_json::from_str(&golden).unwrap();
        assert_eq!(actual, expected, "{} golden drifted", case.id);
        assert_eq!(
            contract.schema_fingerprint,
            *case
                .schema_fingerprint
                .as_ref()
                .expect("a resolved case names its fingerprint"),
            "{} fingerprint drifted",
            case.id
        );
    }
}

#[test]
fn indexed_contract_artifacts_are_readable_by_a_downstream_host() {
    let index = index();
    let mut formats = BTreeSet::new();
    for case in cases_of(&index, "contract-artifact") {
        let golden = read(case.golden.as_deref().expect("an artifact case names its golden"));
        let artifact = match ContractArtifact::from_json(&golden) {
            Ok(artifact) => artifact,
            Err(error) => panic!("{} artifact does not read: {}", case.id, error.code),
        };
        assert_eq!(
            artifact.fingerprint.len(),
            64,
            "{} does not carry a SHA-256 fingerprint",
            case.id
        );
        formats.insert(artifact.version.as_str().to_owned());
    }
    let expected = ["vos/iris.contract/v0", "vos/hermes.contract/v0"]
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    assert_eq!(formats, expected, "an indexed product format is missing");
}

#[test]
fn indexed_operation_sources_keep_their_leading_attributes() {
    let index = index();
    let cases = cases_of(&index, "operation-projection");
    assert!(!cases.is_empty(), "no operation case is indexed");
    for case in cases {
        let operations = parse_oak(&read(&case.entry))
            .expect("Oak parses the indexed source")
            .project_operations()
            .expect("the indexed source projects operations");
        assert_eq!(
            operations.len(),
            case.operations.expect("the case records its operation count"),
            "{} projected a different number of operations",
            case.id
        );
        for operation in &operations {
            assert!(
                !operation.attributes.is_empty(),
                "{} dropped the attributes of {}",
                case.id,
                operation.name
            );
        }
    }
}

#[test]
fn indexed_sub_indexes_declare_their_own_version() {
    let index = index();
    for (kind, version) in [
        ("fingerprint-evolution", "vos-fingerprint-evolution-v0"),
        ("diagnostic-golden", "vos-diagnostic-golden-v0"),
        ("feature-matrix", "vos-feature-matrix-v0"),
    ] {
        let cases = cases_of(&index, kind);
        assert_eq!(cases.len(), 1, "{kind} is catalogued exactly once");
        let nested: Value = serde_json::from_str(&read(&cases[0].entry))
            .unwrap_or_else(|error| panic!("{kind} sub-index does not parse: {error}"));
        assert_eq!(nested["formatVersion"], version, "{kind} sub-index version");
    }
}
