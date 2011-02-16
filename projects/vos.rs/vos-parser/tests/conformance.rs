//! Shared-fixture conformance for VOS parse goldens.
//!
//! Layout: `vos-language/specifications/fixtures/{schema,operations,diagnostics}/`
//! See `specifications/fixtures/README.md`.
//!
//! Bless: `VOS_UPDATE_FIXTURES=1 cargo test -p vos-parser --test conformance`

use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use vos_ast::catalog::{CatalogSnapshot, catalog_from_document};
use vos_ast::expr::Program;
use vos_ast::{Diagnostics, Document};
use vos_parser::{normalize_source, parse_document, parse_program};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ParserKind {
    Document,
    Program,
}

#[derive(Debug, Deserialize)]
struct KindFile {
    parser: ParserKind,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum AstGolden {
    Document(Document),
    Program(Program),
}

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../specifications/fixtures")
}

fn update_fixtures() -> bool {
    matches!(
        std::env::var("VOS_UPDATE_FIXTURES").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE")
    )
}

fn discover_stems(dir: &Path) -> Vec<PathBuf> {
    let mut stems = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return stems;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("vos") {
            continue;
        }
        // Skip companion normalized sources.
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.ends_with(".normalized.vos"))
        {
            continue;
        }
        stems.push(path.with_extension(""));
    }
    stems.sort();
    stems
}

fn default_kind(rel_dir: &str) -> ParserKind {
    if rel_dir == "operations" {
        ParserKind::Program
    } else {
        ParserKind::Document
    }
}

fn resolve_kind(stem: &Path, rel_dir: &str) -> ParserKind {
    let kind_path = stem.with_extension("kind.json");
    if kind_path.is_file() {
        let text = fs::read_to_string(&kind_path).expect("read kind.json");
        return serde_json::from_str::<KindFile>(&text)
            .unwrap_or_else(|e| panic!("{}: {e}", kind_path.display()))
            .parser;
    }
    default_kind(rel_dir)
}

fn write_pretty_json(path: &Path, value: &impl Serialize) {
    let mut text = serde_json::to_string_pretty(value).expect("serialize json");
    text.push('\n');
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(path, text).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
}

fn read_or_empty_diagnostics(path: &Path) -> Diagnostics {
    if !path.is_file() {
        return Diagnostics::default();
    }
    let text = fs::read_to_string(path).expect("read diagnostics");
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn run_case(stem: &Path, rel_dir: &str) {
    let vos_path = stem.with_extension("vos");
    let normalized_path = PathBuf::from(format!("{}.normalized.vos", stem.display()));
    let ast_path = stem.with_extension("ast.json");
    let diag_path = stem.with_extension("diagnostics.json");

    let catalog_path = stem.with_extension("catalog.json");

    let source = fs::read_to_string(&vos_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", vos_path.display()));
    let normalized = normalize_source(&source);
    let kind = resolve_kind(stem, rel_dir);
    let update = update_fixtures();

    if update || !normalized_path.is_file() {
        if update {
            fs::write(&normalized_path, &normalized)
                .unwrap_or_else(|e| panic!("write {}: {e}", normalized_path.display()));
        } else if !normalized_path.is_file() {
            panic!(
                "missing {}; run with VOS_UPDATE_FIXTURES=1",
                normalized_path.display()
            );
        }
    } else {
        let expected = fs::read_to_string(&normalized_path).expect("read normalized");
        assert_eq!(
            expected,
            normalized,
            "normalized source mismatch for {}",
            vos_path.display()
        );
    }

    let (ast_value, diags): (Option<AstGolden>, Diagnostics) = match kind {
        ParserKind::Document => match parse_document(&source) {
            Ok(doc) => (Some(AstGolden::Document(doc)), Diagnostics::default()),
            Err(errors) => (None, errors),
        },
        ParserKind::Program => match parse_program(&source) {
            Ok(program) => (Some(AstGolden::Program(program)), Diagnostics::default()),
            Err(errors) => (None, errors),
        },
    };

    let catalog: Option<CatalogSnapshot> = match &ast_value {
        Some(AstGolden::Document(doc)) => match catalog_from_document(doc) {
            Ok(snap) => Some(snap),
            Err(msg) => panic!("catalog emit failed for {}: {msg}", vos_path.display()),
        },
        _ => None,
    };

    if update {
        write_pretty_json(&diag_path, &diags);
        match &ast_value {
            Some(ast) => write_pretty_json(&ast_path, ast),
            None => {
                if ast_path.is_file() {
                    fs::remove_file(&ast_path).ok();
                }
            }
        }
        match &catalog {
            Some(snap) => write_pretty_json(&catalog_path, snap),
            None => {
                if catalog_path.is_file() {
                    fs::remove_file(&catalog_path).ok();
                }
            }
        }
        return;
    }

    let expected_diags = read_or_empty_diagnostics(&diag_path);
    assert_eq!(
        expected_diags,
        diags,
        "diagnostics mismatch for {}",
        vos_path.display()
    );

    match ast_value {
        Some(ast) => {
            if !ast_path.is_file() {
                panic!(
                    "missing {}; run with VOS_UPDATE_FIXTURES=1",
                    ast_path.display()
                );
            }
            let expected_text = fs::read_to_string(&ast_path).expect("read ast");
            let actual_text = {
                let mut t = serde_json::to_string_pretty(&ast).expect("serialize ast");
                t.push('\n');
                t
            };
            assert_eq!(
                expected_text,
                actual_text,
                "AST mismatch for {}",
                vos_path.display()
            );
        }
        None => {
            if ast_path.is_file() {
                panic!(
                    "unexpected AST golden present for failing case {}",
                    vos_path.display()
                );
            }
            assert!(
                !diags.is_empty(),
                "parse failed with empty diagnostics for {}",
                vos_path.display()
            );
        }
    }

    match catalog {
        Some(snap) => {
            // Catalog goldens are required for successful schema documents under
            // `schema/` and optional elsewhere when the companion exists.
            let require = rel_dir == "schema" || catalog_path.is_file();
            if require {
                if !catalog_path.is_file() {
                    panic!(
                        "missing {}; run with VOS_UPDATE_FIXTURES=1",
                        catalog_path.display()
                    );
                }
                let expected_text = fs::read_to_string(&catalog_path).expect("read catalog");
                let actual_text = {
                    let mut t = serde_json::to_string_pretty(&snap).expect("serialize catalog");
                    t.push('\n');
                    t
                };
                assert_eq!(
                    expected_text,
                    actual_text,
                    "catalog IR mismatch for {}",
                    vos_path.display()
                );
            }
        }
        None => {
            if catalog_path.is_file() {
                panic!(
                    "unexpected catalog golden for non-document case {}",
                    vos_path.display()
                );
            }
        }
    }
}

#[test]
fn conformance_fixtures() {
    let root = fixtures_root();
    assert!(root.is_dir(), "fixtures root missing: {}", root.display());

    let mut cases = 0usize;
    for rel in ["schema", "operations", "diagnostics"] {
        let dir = root.join(rel);
        if !dir.is_dir() {
            continue;
        }
        for stem in discover_stems(&dir) {
            run_case(&stem, rel);
            cases += 1;
        }
    }
    assert!(
        cases > 0,
        "no *.vos fixtures discovered under {}",
        root.display()
    );
}
