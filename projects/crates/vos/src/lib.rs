//! Stable public facade for **VOS — Virtual Object Schema**.
//!
//! Oak owns VOS lexing and parsing. Artifact generation uses Dejavu
//! (`vos-generator`, AOT-preferred).
//!
//! New hosts such as YYDB, YYDS and Iris should call [`parse_oak`] and consume
//! the resolved contract APIs below. The legacy parser facade remains only for
//! migration and must not gain syntax features.

#![warn(missing_docs)]

pub use vos_ast as ast;
pub use vos_contract as contract;
pub use vos_generator as generator;
pub use vos_inspect as inspect;
pub use vos_parser as parser;

/// Field-identity catalog APIs from a parsed document.
pub use vos_ast::{catalog_from_document, evolve_catalog, schema_fingerprint, schema_fingerprint_from_document};
/// Parse source through Oak and return the parser-free VOS contract input.
pub use vos_contract::parse_oak;

/// Parses and validates a schema through Oak and the VOS schema contract.
pub fn validate_schema(source: &str) -> Result<vos_contract::SchemaProjection, String> {
    let input = parse_oak(source)?;
    let projection = input
        .project_schema()
        .map_err(|diagnostics| format!("VOS semantic projection failed: {diagnostics:?}"))?;
    for item in &projection.types {
        if item.kind == vos_contract::TypeContractKind::Table {
            let primary_count = item
                .fields
                .iter()
                .flat_map(|field| field.attributes.iter())
                .filter(|attribute| attribute.name == "primary")
                .count();
            if primary_count != 1 {
                return Err(format!(
                    "table `{}` requires exactly one primary field",
                    item.canonical_path.join("::")
                ));
            }
        }
    }
    Ok(projection)
}
/// Bind an explicit identity manifest to an Oak projection.
pub use vos_contract::bind_identity;
/// Build the strict resolved VOS contract consumed by downstream hosts.
pub use vos_contract::resolve_contract;
/// Strict resolved contract artifact.
pub use vos_contract::ResolvedContract;
/// Resolve names across already-bound source units.
pub use vos_contract::resolve_identity_units;
/// Normalize source bytes before parse / conformance (`*.normalized.vos`).
pub use vos_parser::normalize_source;
/// Parse a VOS expression / operation program (see `docs/operations.md`).
pub use vos_parser::parse_program;
/// Attach source provenance to AST diagnostics (miette).
pub use vos_parser::{VosError, report_diagnostic, report_diagnostics};

pub mod uuid;
pub use uuid::{is_v7 as uuid_is_v7, uuid};

#[cfg(test)]
mod tests {
    use super::validate_schema;

    #[test]
    fn schema_validation_uses_oak_and_requires_table_primary() {
        assert!(validate_schema("table User { @@id: uuid }").is_ok());
        assert!(validate_schema("table User { id: uuid }").is_err());
        assert!(validate_schema("table User {").is_err());
    }
}
