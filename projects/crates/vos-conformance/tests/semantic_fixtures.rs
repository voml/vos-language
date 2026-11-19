//! Fixture-driven checks that live downstream: the Boost semantic module
//! keeps only inline smoke tests, while golden artifacts and the full `data.text`
//! acceptance walk run here against the pinned core.

use std::{
    fs,
    path::{Path, PathBuf},
};

use schema_input_vos::{ServiceProjectionArtifact, parse_schema, syntax::parse};

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn source_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = fs::read_dir(dir)
        .expect("fixture directory")
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
fn service_artifact_fixture_matches_reviewed_golden() {
    let source = include_str!("fixtures/services/basic.vos");
    let golden = include_str!("fixtures/services/basic.contract.json");
    let input = parse_schema(source).expect("`data.text` parses service fixture");
    let artifact = input
        .project_services_artifact()
        .expect("VOS projects supported service fixture");
    let actual: serde_json::Value = serde_json::from_str(&artifact.to_json().unwrap()).unwrap();
    let expected: serde_json::Value = serde_json::from_str(golden).unwrap();
    assert_eq!(actual, expected);
    assert_eq!(
        ServiceProjectionArtifact::from_json(golden).unwrap(),
        artifact
    );
}

#[test]
fn text_accepts_schema_and_operation_fixtures() {
    let root = fixtures_root();
    let mut count = 0usize;
    for category in ["schema", "operations", "services"] {
        for path in source_files(&root.join(category)) {
            let source = fs::read_to_string(&path).expect("fixture source");
            parse(&source)
                .unwrap_or_else(|error| panic!("`data.text` rejected {}: {error}", path.display()));
            count += 1;
        }
    }
    assert!(count > 0, "`data.text` fixture set is empty");
}
