//! Multi-file `using` resolution — **collect pass**.
//!
//! This module walks an Oak-built AST root, extracts `using` paths in both
//! supported surface forms (namespace-style and string-literal), and collects
//! every top-level declaration into a [`SourceDecls`] bundle. The resulting
//! bundles live in a [`SymbolTable`] keyed by source path; a subsequent pass
//! (not included here) feeds `using` paths through a [`Vfs`] and links
//! dependent sources together.

use std::collections::HashMap;
use std::path::PathBuf;

use oak_vos::{VosDeclaration, VosDeclarationKind, VosSyntaxElement};

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
                if let Some(path) =
                    extract_using_path_with_literals(declaration, &collect_string_literals(root))
                {
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
    if let Some(path) = &declaration.path {
        if !path.is_empty() {
            return Some(path.join("/"));
        }
    }

    let decl_start = declaration.span.start;
    let decl_end = declaration.span.end;
    literals
        .iter()
        .find(|(s, e, _)| *s >= decl_start && *e <= decl_end)
        .map(|(_, _, text)| text.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
