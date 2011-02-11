//! Stable public facade for **VOS — Virtual Object Schema**.
//!
//! Parser details remain in `vos-parser`; syntax structures remain in
//! `vos-ast`. Artifact generation uses Dejavu (`vos-generator`, AOT-preferred).
//!
//! Hosts such as YYDB should call [`parser::parse_document`] and map failures
//! with [`parser::report_diagnostics`] (miette + `NamedSource`) so every
//! language error traces back to the originating source span. Do not invent a
//! parallel schema dialect.

#![warn(missing_docs)]

pub use vos_ast as ast;
pub use vos_generator as generator;
pub use vos_inspect as inspect;
pub use vos_parser as parser;

/// Initial field-identity catalog IR from a parsed document.
pub use vos_ast::catalog_from_document;
/// Normalize source bytes before parse / conformance (`*.normalized.vos`).
pub use vos_parser::normalize_source;
/// Parse a VOS expression / operation program (see `docs/operations.md`).
pub use vos_parser::parse_program;
/// Attach source provenance to AST diagnostics (miette).
pub use vos_parser::{VosError, report_diagnostic, report_diagnostics};
