//! Hand-written lexer and parser boundary for **VOS — Virtual Object Schema**.
//!
//! Grammar production is intentionally deferred until the versioned VOS schema
//! contract and conformance fixtures are available. Keeping this as a separate
//! crate prevents AST types and parser implementation details from leaking
//! through the public facade.

/// The AST crate consumed by the future VOS parser.
pub use vos_ast;
