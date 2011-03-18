//! VOS semantic consumer boundary for Oak-owned syntax trees.

#![warn(missing_docs)]

use oak_vos::{VosDeclaration, VosField, VosRoot, VosSyntaxNode};

mod projection;

pub use projection::{ArtifactError, AttributeContract, CANONICALIZATION_VERSION, CanonicalType, CanonicalTypeArgument, ContractDiagnostic, ContractEnvelope, CONTRACT_FORMAT_VERSION, FieldContract, LANGUAGE_VERSION, PROJECTION_STAGE, SchemaProjection, SourceUnit, TypeContract, TypeContractKind};

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

    /// Returns Oak-projected fields without reparsing the source.
    pub fn fields<'a>(&self, declaration: &'a VosDeclaration) -> &'a [VosField] {
        &declaration.fields
    }

    /// Consumes the wrapper and returns the Oak root.
    pub fn into_oak(self) -> VosRoot {
        self.root
    }

    /// Projects the Oak root into the first VOS schema artifact.
    pub fn project_schema(&self) -> Result<SchemaProjection, Vec<ContractDiagnostic>> {
        projection::project_schema(&self.root)
    }
}

#[cfg(test)]
mod tests {
    use super::parse_oak;
    use super::{CANONICALIZATION_VERSION, CanonicalType, CONTRACT_FORMAT_VERSION, LANGUAGE_VERSION, PROJECTION_STAGE};
    use oak_vos::VosDeclarationKind;

    #[test]
    fn consumes_oak_root_without_reparsing() {
        let input = parse_oak("table User { @@id: uuid, email: utf8? = null, }").expect("Oak parses VOS");

        assert_eq!(input.declarations().len(), 1);
        assert_eq!(input.declarations()[0].kind, VosDeclarationKind::Table);
        let fields = input.fields(&input.declarations()[0]);
        assert_eq!(fields[0].name, "id");
        assert_eq!(fields[0].attributes[0].text, "@@");
        assert_eq!(fields[0].type_syntax.text, "uuid");
        assert_eq!(fields[1].type_syntax.text, "utf8?");
        assert_eq!(fields[1].default_value.as_ref().unwrap().text, "null");
        assert!(input.syntax().span.end > input.syntax().span.start);
    }

    #[test]
    fn consumes_oak_field_spans_without_reparsing_source() {
        let source = "class User { [primary] id: uuid, manager: &User?, }";
        let input = parse_oak(source).expect("Oak parses VOS");
        let fields = input.fields(&input.declarations()[0]);
        assert_eq!(&source[fields[0].name_span.clone()], "id");
        assert_eq!(&source[fields[0].type_syntax.span.clone()], "uuid");
        assert_eq!(&source[fields[1].type_syntax.span.clone()], "&User?");
        assert_eq!(&source[fields[0].attributes[0].span.clone()], "[primary]");
    }

    #[test]
    fn projects_oak_output_into_versioned_artifact_without_reparsing() {
        let input = parse_oak("namespace demo::identity\ntable User { @@id: uuid, manager: &User?, tags: [utf8]? = null, }").expect("Oak parses VOS");
        let contract = input.project_schema().expect("VOS projects Oak output");

        assert_eq!(contract.envelope.contract_format_version, CONTRACT_FORMAT_VERSION);
        assert_eq!(contract.envelope.stage, PROJECTION_STAGE);
        assert_eq!(contract.envelope.language_version, LANGUAGE_VERSION);
        assert_eq!(contract.envelope.canonicalization_version, CANONICALIZATION_VERSION);
        assert_eq!(contract.envelope.schema_fingerprint, None);
        assert_eq!(contract.envelope.source_units.len(), 1);
        assert_eq!(contract.types[0].canonical_path, ["demo", "identity", "User"]);
        assert_eq!(contract.types[0].fields[0].attributes[0].name, "primary");
        assert!(matches!(&contract.types[0].fields[1].canonical_type, CanonicalType::Optional(inner) if matches!(inner.as_ref(), CanonicalType::Reference(_))));
        assert!(matches!(&contract.types[0].fields[2].canonical_type, CanonicalType::Optional(inner) if matches!(inner.as_ref(), CanonicalType::List(_))));
        assert_eq!(contract.types[0].fields[2].default_value.as_deref(), Some("null"));
    }

    #[test]
    fn projection_reports_duplicate_canonical_types() {
        let source = "namespace demo\ntable User { id: uuid, }\ntable User { id: uuid, }";
        let input = parse_oak(source).expect("Oak parses duplicate declarations");
        let diagnostics = input.project_schema().expect_err("duplicate type must be diagnosed");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "VOS002");
        assert_eq!(diagnostics[0].span.as_ref().unwrap().start, source.rfind("table User").unwrap());
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
