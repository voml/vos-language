//! Multi-file `using` resolution.
//!
//! Two cooperating passes live here:
//!
//! * **Collect pass** ([`collect_decls`]) walks an Oak-built AST root,
//!   extracts `using` paths in both supported surface forms (namespace-style
//!   and string-literal), and bundles every top-level declaration into a
//!   [`SourceDecls`].
//! * **Resolve pass** ([`resolve_using_tree`]) feeds each collected `using`
//!   path through a [`Vfs`], loads the target source, collects its
//!   declarations, and recurses — with explicit cycle detection.
//!
//! The end result is a fully-populated [`SymbolTable`] keyed by absolute
//! source path.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use oak_vos::{VosDeclaration, VosDeclarationKind, VosSyntaxElement};

use crate::vfs::{Vfs, VfsError};

/// Identifier for a single loaded source unit.
pub type SourceId = PathBuf;

/// Kinds of top-level declarations that VOS tracks in its symbol table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeclKind {
    /// `table Name` or `class Name`.
    Type {
        /// Declaration kind: `Table` or `Class`.
        kind: VosDeclarationKind,
        /// Name of the type.
        name: String,
    },
    /// `service Name`.
    Service {
        /// Name of the service.
        name: String,
    },
    /// `query|udf|micro|macro Name`.
    Operation {
        /// Operation kind.
        kind: VosDeclarationKind,
        /// Name of the operation.
        name: String,
    },
    /// `const Name`.
    Const {
        /// Name of the constant.
        name: String,
    },
    /// `namespace a::b`.
    Namespace {
        /// Qualified path segments.
        path: Vec<String>,
    },
    // `using` is tracked separately via SourceDecls::using_paths.
}

/// All declarations and `using` paths collected from a single source.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SourceDecls {
    /// Namespace currently active within this source.
    pub namespace: Vec<String>,
    /// Namespace-qualified declarations.
    pub declarations: Vec<DeclKind>,
    /// Raw `using` paths as written (namespace segments joined or string
    /// literal stripped).
    pub using_paths: Vec<String>,
}

/// Symbol table for a set of source units.
#[derive(Clone, Debug, Default)]
pub struct SymbolTable {
    /// Loaded source units keyed by absolute path.
    pub sources: HashMap<SourceId, SourceDecls>,
}

impl SymbolTable {
    /// Creates an empty symbol table.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts collected declarations for a source.
    pub fn insert(&mut self, id: SourceId, decls: SourceDecls) {
        self.sources.insert(id, decls);
    }

    /// Returns whether a source has been loaded.
    pub fn contains(&self, id: &SourceId) -> bool {
        self.sources.contains_key(id)
    }
}

/// Walks an Oak-built root and collects declarations plus using paths.
///
/// This is the **collect pass** — no file I/O happens here, we just
/// examine what is already in memory.
pub fn collect_decls(root: &oak_vos::VosRoot) -> SourceDecls {
    let mut decls = SourceDecls::default();
    let literals = collect_string_literals(root);

    for declaration in &root.declarations {
        match declaration.kind {
            VosDeclarationKind::Namespace => {
                decls.namespace = declaration.path.clone().unwrap_or_default();
                decls.declarations.push(DeclKind::Namespace {
                    path: declaration.path.clone().unwrap_or_default(),
                });
            }
            VosDeclarationKind::Table | VosDeclarationKind::Class => {
                if let Some(name) = &declaration.name {
                    decls.declarations.push(DeclKind::Type {
                        kind: declaration.kind,
                        name: name.clone(),
                    });
                }
            }
            VosDeclarationKind::Service => {
                if let Some(name) = &declaration.name {
                    decls
                        .declarations
                        .push(DeclKind::Service { name: name.clone() });
                }
            }
            VosDeclarationKind::Query
            | VosDeclarationKind::Udf
            | VosDeclarationKind::Micro
            | VosDeclarationKind::Macro => {
                if let Some(name) = &declaration.name {
                    decls.declarations.push(DeclKind::Operation {
                        kind: declaration.kind,
                        name: name.clone(),
                    });
                }
            }
            VosDeclarationKind::Const => {
                if let Some(name) = &declaration.name {
                    decls
                        .declarations
                        .push(DeclKind::Const { name: name.clone() });
                }
            }
            VosDeclarationKind::Using => {
                if let Some(path) = extract_using_path_with_literals(declaration, &literals) {
                    decls.using_paths.push(path);
                }
            }
            _ => { /* Enums, Flags, Obsolete — not tracked for resolution */ }
        }
    }

    decls
}

/// (span_start, span_end, unquoted_text) for every StringLiteral in the root.
fn collect_string_literals(root: &oak_vos::VosRoot) -> Vec<(usize, usize, String)> {
    let mut out = Vec::new();
    walk_literals(&root.syntax, &mut out);
    out
}

fn walk_literals(node: &oak_vos::VosSyntaxNode, out: &mut Vec<(usize, usize, String)>) {
    for child in &node.children {
        match child {
            VosSyntaxElement::Token(token) => {
                if matches!(token.kind, oak_vos::VosTokenType::StringLiteral) {
                    let text = token.text.trim();
                    let stripped = text
                        .strip_prefix('"')
                        .or_else(|| text.strip_prefix('\''))
                        .unwrap_or(text);
                    let stripped = stripped
                        .strip_suffix('"')
                        .or_else(|| stripped.strip_suffix('\''))
                        .unwrap_or(stripped);
                    out.push((token.span.start, token.span.end, stripped.to_owned()));
                }
            }
            VosSyntaxElement::Node(inner) => walk_literals(inner, out),
        }
    }
}

/// Extracts the import string from a `Using` declaration.
///
/// Oak builder handles two surface forms:
/// * `using foo::bar` → `declaration.path = ["foo", "bar"]`.
/// * `using "./foo.vos"` → `declaration.path = []`, the literal lives in
///   the CST as a [`VosTokenType::StringLiteral`] token.
///
/// Namespace-style takes precedence; otherwise we locate the StringLiteral
/// whose span is contained inside the declaration span.
pub fn extract_using_path_with_literals(
    declaration: &VosDeclaration,
    literals: &[(usize, usize, String)],
) -> Option<String> {
    if let Some(path) = &declaration.path && !path.is_empty() {
        return Some(path.join("/"));
    }

    let decl_start = declaration.span.start;
    let decl_end = declaration.span.end;
    literals
        .iter()
        .find(|(s, e, _)| *s >= decl_start && *e <= decl_end)
        .map(|(_, _, text)| text.clone())
}

/// Resolves all `using` declarations transitively.
///
/// Starts from `entry_path`, reads each source through [`Vfs`], collects
/// declarations, and recurses into every `using` path it finds. A simple
/// `Vec<PathBuf>` visitor stack detects import cycles before they loop.
///
/// On success returns the fully-populated [`SymbolTable`]. If any I/O or
/// cycle error was encountered, the partial table is returned together with
/// the collected [`VfsError`] list — callers can decide whether to abort or
/// continue with the best-effort result.
pub fn resolve_using_tree<V: Vfs>(
    entry_path: &Path,
    vfs: &V,
) -> Result<SymbolTable, (SymbolTable, Vec<VfsError>)> {
    let mut table = SymbolTable::new();
    let mut diagnostics = Vec::new();
    let mut visited: Vec<PathBuf> = Vec::new();

    resolve_recursive(entry_path, vfs, &mut table, &mut visited, &mut diagnostics);

    if diagnostics.is_empty() {
        Ok(table)
    } else {
        Err((table, diagnostics))
    }
}

fn resolve_recursive<V: Vfs>(
    current: &Path,
    vfs: &V,
    table: &mut SymbolTable,
    visited: &mut Vec<PathBuf>,
    diagnostics: &mut Vec<VfsError>,
) {
    // Canonicalize up-front so table keys and cycle checks are stable even
    // when callers pass relative paths.
    let canonical = current
        .canonicalize()
        .unwrap_or_else(|_| current.to_path_buf());

    // Cycle detection: is `canonical` already on our import stack?
    if let Some(pos) = visited.iter().position(|p| p == &canonical) {
        let cycle: Vec<String> = visited[pos..]
            .iter()
            .map(|p| p.display().to_string())
            .chain(std::iter::once(canonical.display().to_string()))
            .collect();
        diagnostics.push(VfsError::circular(cycle.join(" -> ")));
        return;
    }

    // Memoized: already fully processed?
    if table.contains(&canonical) {
        return;
    }

    let source = match vfs.read_file(&canonical) {
        Ok(s) => s,
        Err(e) => {
            diagnostics.push(e);
            return;
        }
    };

    let root = match oak_vos::parse(&source) {
        Ok(r) => r,
        Err(e) => {
            diagnostics.push(VfsError::new(
                crate::vfs::VfsErrorKind::Io,
                format!("parse error in {}: {e}", canonical.display()),
            ));
            return;
        }
    };

    let decls = collect_decls(&root);
    let using_paths = decls.using_paths.clone();
    table.insert(canonical.clone(), decls);

    visited.push(canonical.clone());
    for import in using_paths {
        let resolved = match vfs.resolve(&canonical, &import) {
            Ok(p) => p,
            Err(e) => {
                diagnostics.push(e);
                continue;
            }
        };
        resolve_recursive(&resolved, vfs, table, visited, diagnostics);
    }
    visited.pop();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vfs::FsVfs;
    use std::fs;

    #[test]
    fn collects_namespace_style_using() {
        let root = oak_vos::parse("using foo::bar").unwrap();
        let decls = collect_decls(&root);
        assert_eq!(decls.using_paths, vec!["foo/bar".to_owned()]);
    }

    #[test]
    fn collects_string_literal_using() {
        let root = oak_vos::parse("using \"./shared.vos\"").unwrap();
        let decls = collect_decls(&root);
        assert_eq!(decls.using_paths, vec!["./shared.vos".to_owned()]);
    }

    #[test]
    fn collects_decls_under_namespace() {
        let root = oak_vos::parse(
            "namespace demo\ntable User { @@id: uuid, }\nservice Identity { create() }",
        )
        .unwrap();
        let decls = collect_decls(&root);
        assert_eq!(decls.namespace, vec!["demo".to_owned()]);
        assert_eq!(decls.declarations.len(), 3); // Namespace + Table + Service
    }

    #[test]
    fn empty_source_produces_empty_decls() {
        let root = oak_vos::parse("").unwrap();
        let decls = collect_decls(&root);
        assert!(decls.declarations.is_empty());
        assert!(decls.using_paths.is_empty());
    }

    #[test]
    fn symbol_table_insert_and_contains() {
        let mut table = SymbolTable::new();
        let id = PathBuf::from("main.vos");
        let decls = SourceDecls {
            namespace: vec!["demo".to_owned()],
            declarations: vec![DeclKind::Type {
                kind: VosDeclarationKind::Table,
                name: "User".to_owned(),
            }],
            using_paths: vec!["./shared.vos".to_owned()],
        };
        assert!(!table.contains(&id));
        table.insert(id.clone(), decls.clone());
        assert!(table.contains(&id));
        assert_eq!(table.sources.get(&id).unwrap(), &decls);
    }

    // — Resolve pass tests (require actual filesystem) —

    #[test]
    fn resolves_single_using() {
        let dir = tempfile_dir();
        let main = dir.join("main.vos");
        let shared = dir.join("shared.vos");
        fs::write(
            &main,
            "using \"./shared.vos\"\n\ntable Main { @@id: uuid, }",
        )
        .unwrap();
        fs::write(&shared, "table Shared { @@id: uuid, }").unwrap();

        let vfs = FsVfs::new();
        let table = resolve_using_tree(&main, &vfs).expect("no errors");

        assert_eq!(table.sources.len(), 2);
        let main_key = main.canonicalize().unwrap();
        let shared_key = shared.canonicalize().unwrap();
        assert!(table.sources.contains_key(&main_key));
        assert!(table.sources.contains_key(&shared_key));
        assert_eq!(
            table.sources.get(&main_key).unwrap().using_paths,
            vec!["./shared.vos".to_owned()]
        );
    }

    #[test]
    fn resolves_transitive_using() {
        let dir = tempfile_dir();
        let a = dir.join("a.vos");
        let b = dir.join("b.vos");
        let c = dir.join("c.vos");
        fs::write(&a, "using \"./b.vos\"\ntable A { @@id: uuid, }").unwrap();
        fs::write(&b, "using \"./c.vos\"\ntable B { @@id: uuid, }").unwrap();
        fs::write(&c, "table C { @@id: uuid, }").unwrap();

        let vfs = FsVfs::new();
        let table = resolve_using_tree(&a, &vfs).expect("no errors");
        assert_eq!(table.sources.len(), 3);
    }

    #[test]
    fn circular_using_is_detected() {
        let dir = tempfile_dir();
        let a = dir.join("a.vos");
        let b = dir.join("b.vos");
        fs::write(&a, "using \"./b.vos\"").unwrap();
        fs::write(&b, "using \"./a.vos\"").unwrap();

        let vfs = FsVfs::new();
        let err = resolve_using_tree(&a, &vfs).expect_err("circular must report");
        let (_partial, diagnostics) = err;
        assert!(
            diagnostics
                .iter()
                .any(|d| d.kind == crate::vfs::VfsErrorKind::Circular)
        );
    }

    #[test]
    fn self_using_is_detected_as_circular() {
        let dir = tempfile_dir();
        let a = dir.join("a.vos");
        fs::write(&a, "using \"./a.vos\"").unwrap();

        let vfs = FsVfs::new();
        let err = resolve_using_tree(&a, &vfs).expect_err("self-import must report");
        let (_partial, diagnostics) = err;
        assert!(
            diagnostics
                .iter()
                .any(|d| d.kind == crate::vfs::VfsErrorKind::Circular)
        );
    }

    #[test]
    fn missing_import_is_reported() {
        let dir = tempfile_dir();
        let a = dir.join("a.vos");
        fs::write(&a, "using \"./does-not-exist.vos\"").unwrap();

        let vfs = FsVfs::new();
        let err = resolve_using_tree(&a, &vfs).expect_err("missing import must report");
        let (_partial, diagnostics) = err;
        assert!(
            diagnostics
                .iter()
                .any(|d| d.kind == crate::vfs::VfsErrorKind::NotFound)
        );
    }

    fn tempfile_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "vos-resolve-tests-{}-{}",
            std::process::id(),
            rand_suffix()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn rand_suffix() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_string()
    }
}
