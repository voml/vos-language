//! Stable public facade for **VOS — Virtual Object Schema**.
//!
//! Oak owns VOS lexing, parsing, CST and syntax projection (`oak-vos`). VOS owns
//! name resolution, canonical types, durable identity, the resolved contract and
//! structured diagnostics, and exposes them through [`vos-contract`] here.
//! Artifact generation uses Dejavu (`vos-generator`, AOT-preferred).
//!
//! Hosts such as YYDB, YYDS and Iris must call [`parse_oak`] and consume the
//! resolved contract APIs below. This crate never owns a parser, an AST, a CST
//! or a lexical layer of its own.

#![warn(missing_docs)]

pub use vos_contract as contract;
pub use vos_format as format;
pub use vos_generator as generator;

/// Strict resolved contract artifact.
pub use vos_contract::ResolvedContract;
/// Bind an explicit identity manifest to an Oak projection.
pub use vos_contract::bind_identity;
/// Parse source through Oak and return the parser-free VOS contract input.
pub use vos_contract::parse_oak;
/// Build the strict resolved VOS contract consumed by downstream hosts.
pub use vos_contract::resolve_contract;
/// Resolve names across already-bound source units.
pub use vos_contract::resolve_identity_units;

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

pub mod uuid;
pub use uuid::{is_v7 as uuid_is_v7, uuid};
