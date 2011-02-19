//! Expression / program AST for VOS operations.
//!
//! Normative surface: `vos-language/docs/expressions.md`.

use crate::{Literal, Span, TypeExpr};
use serde::{Deserialize, Serialize};

/// Top-level script / query program: statements then optional result expression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    /// Session-local `micro` declarations (not catalog; die with the session).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub micros: Vec<FnDecl>,
    /// Leading statements (`let`, …).
    pub statements: Vec<Stmt>,
    /// Optional trailing expression (program result).
    pub result: Option<Expr>,
    /// Whole-program span when known.
    pub span: Span,
}

/// `micro` (session) or `macro` (durable DDL) function declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FnDecl {
    /// `micro` vs `macro`.
    pub kind: FnKind,
    /// Function name.
    pub name: String,
    /// Parameters.
    pub params: Vec<FnParam>,
    /// Optional return type (`-> Type`).
    pub return_ty: Option<TypeExpr>,
    /// Body as a nested program (`{ … }`).
    pub body: Program,
    /// Declaration span including the closing `}`.
    pub span: Span,
}

/// Kind of function declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FnKind {
    /// Session-local temporary function.
    Micro,
    /// Durable DDL catalog function.
    Macro,
}

/// One parameter: `name: Type`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FnParam {
    /// Parameter name.
    pub name: String,
    /// Parameter type.
    pub ty: TypeExpr,
    /// Span of `name: Type`.
    pub span: Span,
}

/// One program statement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Stmt {
    /// Immutable `let` binding.
    Let(Let),
    /// Expression used as a statement (rare in v1).
    Expr(Expr),
}

/// `let name [: Type] = expression`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Let {
    /// Bound name.
    pub name: String,
    /// Optional type annotation.
    pub ty: Option<TypeExpr>,
    /// Initializer expression.
    pub value: Expr,
    /// Declaration span.
    pub span: Span,
}

/// Shared field initializer: bare `name` or `name: expression`.
///
/// Used by typed objects, anonymous structs, projections, and patches.
/// **Not** `expression as name`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldInit {
    /// Output / field name (left of `:`).
    pub name: String,
    /// `None` means shorthand (`name` ≡ `name: <context>.name`).
    pub value: Option<Expr>,
    /// Source span of this initializer.
    pub span: Span,
}

impl FieldInit {
    /// Bare-field shorthand.
    pub fn shorthand(name: impl Into<String>, span: Span) -> Self {
        Self {
            name: name.into(),
            value: None,
            span,
        }
    }

    /// Explicit `name: expression`.
    pub fn named(name: impl Into<String>, value: Expr, span: Span) -> Self {
        Self {
            name: name.into(),
            value: Some(value),
            span,
        }
    }

    /// True when written as bare `name` without `:`.
    pub fn is_shorthand(&self) -> bool {
        self.value.is_none()
    }
}

/// Item inside `x.{ … }` (or anonymous `{ … }` when used as projection body).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ProjItem {
    /// Spread all public fields of the receiver (`*`).
    Star {
        /// Span of `*`.
        span: Span,
    },
    /// `name` or `name: expression`.
    Field(FieldInit),
}

impl ProjItem {
    /// Output field name contributed by this item, if any (`*` contributes none alone).
    pub fn output_name(&self) -> Option<&str> {
        match self {
            Self::Star { .. } => None,
            Self::Field(init) => Some(init.name.as_str()),
        }
    }
}

/// Unary operators (v1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum UnaryOp {
    /// `!expr`
    Not,
}

/// Binary operators (v1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum BinaryOp {
    /// `==`
    Eq,
    /// `!=`
    Ne,
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// `>`
    Gt,
    /// `>=`
    Ge,
    /// `&&`
    And,
    /// `||`
    Or,
    /// `+` (numeric or string concat — host typing decides)
    Add,
    /// `-`
    Sub,
    /// `*`
    Mul,
    /// `/`
    Div,
}

/// Lambda: `x => expr` or `(a, b) => expr`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lambda {
    /// Parameter names in order.
    pub params: Vec<String>,
    /// Body expression.
    pub body: Box<Expr>,
    /// Span of the whole lambda.
    pub span: Span,
}

/// Path separator between receiver and member (`a.b` vs `a::b`).
///
/// Both forms are accepted for static symbol paths. Canonical formatters emit
/// `::` for static DDL / namespace / type paths. See `vos-language/docs/paths.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PathSep {
    /// `.` — default value member access; also allowed on static paths.
    #[default]
    Dot,
    /// `::` — recommended static namespace / type / catalog navigation.
    ColonColon,
}

impl PathSep {
    /// Source spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dot => ".",
            Self::ColonColon => "::",
        }
    }

    /// True when this is the recommended static-path separator.
    pub fn is_static_recommended(self) -> bool {
        matches!(self, Self::ColonColon)
    }
}

/// Expression node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Expr {
    /// Literal (shared with schema defaults).
    Literal(Literal),
    /// Local / type / table name.
    Name {
        /// Identifier text.
        name: String,
        /// Span.
        span: Span,
    },
    /// `Type { field: expr, … }` — construction only, no write.
    TypedObject {
        /// Type name (`User`, …).
        ty: String,
        /// Field initializers.
        fields: Vec<FieldInit>,
        /// Span.
        span: Span,
    },
    /// `{ field: expr, … }` anonymous struct.
    AnonObject {
        /// Field initializers.
        fields: Vec<FieldInit>,
        /// Span.
        span: Span,
    },
    /// `[ expr, … ]`.
    List {
        /// Elements.
        items: Vec<Expr>,
        /// Span.
        span: Span,
    },
    /// `x => …` / `(…) => …`.
    Lambda(Lambda),
    /// `expr.ident`.
    Member {
        /// Receiver.
        object: Box<Expr>,
        /// Field / method name (before call).
        name: String,
        /// Source separator (`.` or `::`). Typed IR ignores this; formatters may
        /// rewrite static paths to `::`.
        sep: PathSep,
        /// Span of `.name` / `::name`.
        span: Span,
    },
    /// `callee(args…)` (method or free call).
    Call {
        /// Callee expression (`Name`, `Member`, …).
        callee: Box<Expr>,
        /// Arguments.
        args: Vec<Expr>,
        /// Span of the call including `(…)`.
        span: Span,
    },
    /// `!expr`.
    Unary {
        /// Operator.
        op: UnaryOp,
        /// Operand.
        expr: Box<Expr>,
        /// Span.
        span: Span,
    },
    /// `left op right`.
    Binary {
        /// Operator.
        op: BinaryOp,
        /// Left operand.
        left: Box<Expr>,
        /// Right operand.
        right: Box<Expr>,
        /// Span.
        span: Span,
    },
    /// `expr.*` — full anonymous structural projection.
    StarProj {
        /// Receiver.
        receiver: Box<Expr>,
        /// Span of `.*`.
        span: Span,
    },
    /// `expr.{ items }` — selective projection.
    StructProj {
        /// Receiver.
        receiver: Box<Expr>,
        /// Projection items (`*`, field inits).
        items: Vec<ProjItem>,
        /// Span of `.{ … }`.
        span: Span,
    },
    /// Postfix `expr?` (optional unwrap / error propagate).
    Try {
        /// Operand.
        expr: Box<Expr>,
        /// Span of `?`.
        span: Span,
    },
}

impl Expr {
    /// Convenience: identifier name.
    pub fn name(name: impl Into<String>, span: Span) -> Self {
        Self::Name {
            name: name.into(),
            span,
        }
    }

    /// Convenience: member access with `.`.
    pub fn member(object: Expr, name: impl Into<String>, span: Span) -> Self {
        Self::member_with_sep(object, PathSep::Dot, name, span)
    }

    /// Member access with an explicit separator (`.` or `::`).
    pub fn member_with_sep(
        object: Expr,
        sep: PathSep,
        name: impl Into<String>,
        span: Span,
    ) -> Self {
        Self::Member {
            object: Box::new(object),
            name: name.into(),
            sep,
            span,
        }
    }
}

/// Collect explicit output names from projection items and report duplicates.
///
/// When `spread_fields` is `Some`, each name from `*` is treated as already
/// defined (for `VOS-PROJECTION-0004` checks). Does not expand unknown `*`.
pub fn projection_result_names(
    items: &[ProjItem],
    spread_fields: Option<&[String]>,
) -> Result<Vec<String>, (String, Span)> {
    let mut out = Vec::new();
    let mut from_spread = false;
    for item in items {
        match item {
            ProjItem::Star { .. } => {
                if let Some(fields) = spread_fields {
                    for name in fields {
                        if out.iter().any(|n| n == name) {
                            return Err((name.clone(), Span::empty(0)));
                        }
                        out.push(name.clone());
                    }
                    from_spread = true;
                }
            }
            ProjItem::Field(init) => {
                if out.iter().any(|n| n == &init.name) {
                    let _ = from_spread;
                    return Err((init.name.clone(), init.span));
                }
                out.push(init.name.clone());
            }
        }
    }
    let _ = from_spread;
    Ok(out)
}
