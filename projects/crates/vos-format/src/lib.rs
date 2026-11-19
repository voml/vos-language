//! VOS source formatting on the `data.text`-owned CST.
//!
//! `data.text` owns the VOS lexer, parser, CST, spans and recovery. This crate owns no
//! lexer, no parser and no AST: it consumes `data-text-vos` output only.
//!
//! Two modes are defined by the language contract:
//!
//! - [`Mode::Preserve`] keeps the author's own spelling. It re-emits the exact
//!   source bytes from the lossless CST, so `format_preserving` is a
//!   round-trip check of the `data.text` CST rather than a rewrite.
//! - [`Mode::Canonical`] rewrites layout for the declarations `data.text` decomposes
//!   structurally (`namespace`, `using`, `table`, `class`) and normalizes
//!   static paths to `::`. Declarations `data.text` does not decompose yet
//!   (`enums`, `flags`, `const`, `obsolete`, `service`, `micro`, `macro`,
//!   `query`, `udf`) are passed through verbatim, so canonical mode never loses
//!   source it does not understand.

#![deny(missing_docs)]

mod canonical;
mod preserve;

pub use canonical::format_canonical;
pub use preserve::format_preserving;

/// Formatting mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Keep the author's own spelling (`paths.md`).
    Preserve,
    /// Canonical layout; static paths use `::`.
    Canonical,
}

/// Format VOS source in the requested mode.
pub fn format(source: &str, mode: Mode) -> Result<String, String> {
    match mode {
        Mode::Preserve => format_preserving(source),
        Mode::Canonical => format_canonical(source),
    }
}
