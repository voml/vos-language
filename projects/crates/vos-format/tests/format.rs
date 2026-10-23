//! Formatter conformance: lossless round-trip and canonical layout.

use std::fs;
use std::path::{Path, PathBuf};

use vos_format::{Mode, format, format_canonical, format_preserving};

fn schema_fixtures() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../documentation/fixtures/schema");
    let mut files: Vec<PathBuf> = fs::read_dir(&root)
        .unwrap_or_else(|error| panic!("read schema fixtures at {}: {error}", root.display()))
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("vos"))
        .collect();
    files.sort();
    assert!(
        !files.is_empty(),
        "no schema fixtures discovered at {}",
        root.display()
    );
    files
}

#[test]
fn preserve_round_trips_every_schema_fixture() {
    for path in schema_fixtures() {
        let source = fs::read_to_string(&path).expect("read fixture");
        let restored = format_preserving(&source)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        assert_eq!(
            restored,
            source,
            "preserve lost trivia for {}",
            path.display()
        );
    }
}

#[test]
fn preserve_round_trips_comments_blank_lines_and_tabs() {
    let source = "# leading comment\n\nusing shared::UserId;\n\ntable User {\n\t@@id: uuid, # key\n\tname: utf8,\n}\n";
    assert_eq!(format_preserving(source).unwrap(), source);
}

#[test]
fn preserve_mode_rejects_lexical_errors() {
    assert!(format_preserving("table User { name: utf8; \0 }").is_err());
    assert!(format_preserving("table User {").is_err());
}

#[test]
fn mode_argument_selects_the_same_behaviour() {
    let source = "table User { @@id: uuid }\n";
    assert_eq!(format(source, Mode::Preserve).unwrap(), source);
    assert_eq!(
        format(source, Mode::Canonical).unwrap(),
        "table User {\n    @@id: uuid,\n}\n"
    );
}

#[test]
fn canonical_normalizes_layout_for_namespace_table_and_class() {
    let source = "namespace   demo::identity\n\ntable User {\n        @@id: uuid,\n   name:    utf8 = \"anonymous\",\n      manager:  &shared::User? = null,\n}\n\nclass PublicUser {\n  id: uuid,\n\tname: utf8,\n}\n";
    let expected = "namespace demo::identity\n\ntable User {\n    @@id: uuid,\n    name: utf8 = \"anonymous\",\n    manager: &shared::User? = null,\n}\n\nclass PublicUser {\n    id: uuid,\n    name: utf8,\n}\n";
    assert_eq!(format_canonical(source).unwrap(), expected);
}

#[test]
fn canonical_renders_wrappers_and_generics_without_trivia() {
    let source = "table T {\n  a: vector<3>,\n  b: DateTime<UTC>,\n  c: list<&User>,\n  d: [[utf8]?]?,\n  e: &shared::User # comment\n ?,\n}\n";
    let expected = "table T {\n    a: vector<3>,\n    b: DateTime<UTC>,\n    c: list<&User>,\n    d: [[utf8]?]?,\n    e: &shared::User?,\n}\n";
    assert_eq!(format_canonical(source).unwrap(), expected);
}

#[test]
fn canonical_keeps_author_attribute_spelling_including_arguments() {
    let source = "table Account {\n [primary, wire] [backfill(\"legacy, value\")] id: utf8 = \"\",\n @email: utf8,\n}\n";
    let expected = "table Account {\n    [primary, wire] [backfill(\"legacy, value\")] id: utf8 = \"\",\n    @email: utf8,\n}\n";
    assert_eq!(format_canonical(source).unwrap(), expected);
}

#[test]
fn canonical_passes_undecomposed_declarations_through_verbatim() {
    let source = "namespace demo::x\n\nenums Access { Yes = 1, No = 2, }\n\nobsolete table Dealer;\n\nmicro m(value: utf8) -> utf8 { value }\n";
    assert_eq!(format_canonical(source).unwrap(), source);
}

#[test]
fn canonical_is_idempotent() {
    for path in schema_fixtures() {
        let source = fs::read_to_string(&path).expect("read fixture");
        let once =
            format_canonical(&source).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let twice = format_canonical(&once)
            .unwrap_or_else(|error| panic!("{} (second pass): {error}", path.display()));
        assert_eq!(
            once,
            twice,
            "canonical is not idempotent for {}",
            path.display()
        );
    }
}

#[test]
fn canonical_rejects_lexical_errors() {
    assert!(format_canonical("table User { name: utf8; \0 }").is_err());
}
