//! VOS semantic consumer boundary for Oak-owned syntax trees.

#![warn(missing_docs)]

use oak_vos::{VosDeclaration, VosRoot, VosSyntaxNode};

/// Parses VOS source through Oak and wraps the resulting root for semantic use.
pub fn parse_oak(source: &str) -> Result<ContractInput, String> {
    oak_vos::parse(source).map(ContractInput::from_oak)
}

/// Oak output accepted by the VOS semantic pipeline.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContractInput {
    root: VosRoot,
}

impl ContractInput {
    /// Wraps an Oak-built VOS root without reparsing its source.
    pub fn from_oak(root: VosRoot) -> Self {
        Self { root }
    }

    /// Returns the exact source owned by Oak.
    pub fn source(&self) -> &str {
        &self.root.source
    }

    /// Returns the lossless Oak CST for semantic traversal.
    pub fn syntax(&self) -> &VosSyntaxNode {
        &self.root.syntax
    }

    /// Returns top-level declarations emitted by Oak Builder.
    pub fn declarations(&self) -> &[VosDeclaration] {
        &self.root.declarations
    }

    /// Consumes the wrapper and returns the Oak root.
    pub fn into_oak(self) -> VosRoot {
        self.root
    }
}

#[cfg(test)]
mod tests {
    use super::parse_oak;
    use oak_vos::VosDeclarationKind;

    #[test]
    fn consumes_oak_root_without_reparsing() {
        let input = parse_oak("table User { @@id: uuid, }").expect("Oak parses VOS");

        assert_eq!(input.declarations().len(), 1);
        assert_eq!(input.declarations()[0].kind, VosDeclarationKind::Table);
        assert!(input.syntax().span.end > input.syntax().span.start);
    }
}

#[cfg(test)]
mod fixture_tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use oak_vos::parse;

    fn fixtures_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../specifications/fixtures")
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
    fn oak_accepts_schema_and_operation_fixtures() {
        let root = fixtures_root();
        let mut count = 0usize;
        for category in ["schema", "operations"] {
            for path in source_files(&root.join(category)) {
                let source = fs::read_to_string(&path).expect("fixture source");
                parse(&source).unwrap_or_else(|error| {
                    panic!("Oak rejected {}: {error}", path.display())
                });
                count += 1;
            }
        }
        assert!(count > 0, "Oak fixture set is empty");
    }
}
