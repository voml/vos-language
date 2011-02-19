//! Stable diagnostic codes for expression / projection / operation checks.
//!
//! Message text may be localized; codes are not. See
//! `vos-language/docs/expressions.md` and `docs/operations.md`.

/// Duplicate result field name in one projection.
pub const PROJECTION_0001: &str = "VOS-PROJECTION-0001";
/// Unknown field in bare projection shorthand.
pub const PROJECTION_0002: &str = "VOS-PROJECTION-0002";
/// Projection item is neither `*` nor a field initializer.
pub const PROJECTION_0003: &str = "VOS-PROJECTION-0003";
/// Result field collides with a name from `*` or another item.
pub const PROJECTION_0004: &str = "VOS-PROJECTION-0004";

/// Unknown name in expression.
pub const EXPR_0001: &str = "VOS-EXPR-0001";
/// Type annotation mismatch on `let`.
pub const EXPR_0002: &str = "VOS-EXPR-0002";
/// Illegal assignment / local mutation in v1.
pub const EXPR_0003: &str = "VOS-EXPR-0003";
/// Cross-ref member access cannot be resolved (path / type / host policy).
pub const EXPR_0004: &str = "VOS-EXPR-0004";
/// Ambiguous bare name: outer binding shadows receiver field.
pub const EXPR_0005: &str = "VOS-EXPR-0005";

/// Unknown table / collection receiver.
pub const OP_0001: &str = "VOS-OP-0001";
/// Method not valid on receiver type.
pub const OP_0002: &str = "VOS-OP-0002";
/// Write method on a structural projection.
pub const OP_0003: &str = "VOS-OP-0003";
/// `.update` argument is a projection, not a patch.
pub const OP_0004: &str = "VOS-OP-0004";
/// Execution boundary missing where `[T]` was required.
pub const OP_0005: &str = "VOS-OP-0005";
/// Cross-ref use site cannot be resolved (path / type / host policy).
pub const OP_0007: &str = "VOS-OP-0007";

/// `.*` / reflection / dynamic field access inside a `macro`.
pub const MACRO_DYNAMIC_FORBIDDEN: &str = "VOS-MACRO-DYNAMIC-FORBIDDEN";

/// `macro` declared outside a DDL session (e.g. in a query program).
pub const DDL_SESSION_REQUIRED: &str = "VOS-DDL-SESSION-REQUIRED";

/// DDL call argument has the wrong kind (e.g. field value where field symbol required).
pub const DDL_ARGUMENT_0002: &str = "VOS-DDL-ARGUMENT-0002";

/// Catalog / field symbol used where a runtime value is required.
pub const SYMBOL_0004: &str = "VOS-SYMBOL-0004";
