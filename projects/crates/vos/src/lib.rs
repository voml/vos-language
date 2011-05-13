//! Stable public facade for **VOS — Virtual Object Schema**.
//!
//! Oak owns VOS lexing, parsing, CST and syntax projection (`oak-vos`). VOS owns
//! name resolution, canonical types, durable identity, the resolved contract and
//! structured diagnostics. Foundational models are exposed through [`types`].
//! Artifact generation uses Dejavu (`vos-generator`, AOT-preferred).
//!
//! Hosts such as YYDB, YYDS and Iris must call [`parse_oak`] and consume the
//! resolved contract APIs below. This crate never owns a parser, an AST, a CST
//! or a lexical layer of its own.

#![warn(missing_docs)]

pub use vos_format as format;
pub use vos_generator as generator;
pub use vos_types as types;

mod semantic;
pub use semantic::*;

/// Parses and validates a schema through Oak and the VOS schema contract.
pub fn validate_schema(source: &str) -> Result<vos_types::SchemaProjection, String> {
    let input = parse_oak(source)?;
    let projection = input
        .project_schema()
        .map_err(|diagnostics| format!("VOS semantic projection failed: {diagnostics:?}"))?;
    for item in &projection.types {
        if item.kind == vos_types::TypeContractKind::Table {
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
