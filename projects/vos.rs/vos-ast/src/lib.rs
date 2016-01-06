//! Syntax data types for **VOS — Virtual Object Schema**.
//!
//! This crate deliberately has no parser, executor, migration, or database
//! dependency. Those layers consume a versioned VOS AST once the grammar and
//! diagnostics contract are defined.

/// A byte range in one VOS source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// Inclusive byte offset of the first character.
    pub start: usize,
    /// Exclusive byte offset following the last character.
    pub end: usize,
}
