//! Stable public facade for **VOS — Virtual Object Schema**.
//!
//! Parser details remain in `vos-parser`; syntax structures remain in
//! `vos-ast`. Execution and migration semantics belong to consumers, not to
//! this facade.
//!
//! This crate currently re-exports the upstream Iris implementation while the
//! remote repository migrates to the VOS name.

pub use vos_remote::*;
