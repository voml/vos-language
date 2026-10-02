//! Operation IR (lazy query plans + execution boundaries).
//!
//! Normative surface: `vos-language/docs/operations.md`.
//! Lowered from method-call expression trees; not a SQL AST.
//! No-SQL invariant: VOS product surfaces must not introduce SQL grammar or execution chains.

use crate::Span;
use crate::expr::{Expr, FieldInit, Lambda};

/// Reference to a table collection (`User`, …).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableRef {
    /// Table type name.
    pub name: String,
    /// Span of the name.
    pub span: Span,
}

/// Sort direction for one key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SortDir {
    /// Ascending.
    Asc,
    /// Descending.
    Desc,
}

/// One sort key.
#[derive(Debug, Clone, PartialEq)]
pub struct SortKey {
    /// Key expression (often a lambda body or field access).
    pub expr: Expr,
    /// Direction.
    pub dir: SortDir,
    /// Span.
    pub span: Span,
}

/// One stage in a lazy [`QueryPlan`].
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Stage {
    /// Explicit or implied `Type.all()`.
    All {
        /// Span.
        span: Span,
    },
    /// `.filter(predicate)`.
    Filter {
        /// Predicate lambda / expression.
        predicate: Expr,
        /// Span.
        span: Span,
    },
    /// `.map(projection)`.
    Map {
        /// Projection expression (`StarProj` / `StructProj` / …).
        projection: Expr,
        /// Span.
        span: Span,
    },
    /// `.sort_by` / `.sort_by_desc` / multi-key sort.
    Sort {
        /// Sort keys.
        keys: Vec<SortKey>,
        /// Span.
        span: Span,
    },
    /// `.skip(n)`.
    Skip {
        /// Count expression.
        count: Expr,
        /// Span.
        span: Span,
    },
    /// `.take(n)`.
    Take {
        /// Count expression.
        count: Expr,
        /// Span.
        span: Span,
    },
    /// Optional association `.load(…)` hint (use-site inference is the default).
    Load {
        /// Load selector (lambda or path expression).
        selector: Expr,
        /// Span.
        span: Span,
    },
}

/// Lazy query plan: table source + stages. Does not execute until [`Exec`].
#[derive(Debug, Clone, PartialEq)]
pub struct QueryPlan {
    /// Source table collection.
    pub source: TableRef,
    /// Pipeline stages in order.
    pub stages: Vec<Stage>,
    /// Span of the overall plan when known.
    pub span: Span,
}

/// Patch literal: field → expression (not a [`crate::expr::Expr::StructProj`]).
#[derive(Debug, Clone, PartialEq)]
pub struct Patch {
    /// Fields to update (`name: expr`; shorthand allowed when unambiguous).
    pub fields: Vec<FieldInit>,
    /// Span.
    pub span: Span,
}

/// Execution boundary / write op.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Exec {
    /// Materialize `[T]`.
    Collect {
        /// Plan to run.
        plan: QueryPlan,
        /// Span.
        span: Span,
    },
    /// `Type.get(id)` / plan `.first(…)`.
    First {
        /// Optional plan; `None` when using table `.get` / `.first` sugar.
        plan: Option<QueryPlan>,
        /// Key or predicate argument.
        arg: Expr,
        /// Span.
        span: Span,
    },
    /// `.count()` / `.count(predicate)`.
    Count {
        /// Optional plan.
        plan: Option<QueryPlan>,
        /// Optional predicate.
        predicate: Option<Expr>,
        /// Span.
        span: Span,
    },
    /// `.any(predicate)`.
    Any {
        /// Optional plan.
        plan: Option<QueryPlan>,
        /// Predicate.
        predicate: Expr,
        /// Span.
        span: Span,
    },
    /// `.get(id)` on a table.
    Get {
        /// Table.
        table: TableRef,
        /// Primary key expression.
        key: Expr,
        /// Span.
        span: Span,
    },
    /// `.insert()` on a value or list.
    Insert {
        /// Value / list expression being inserted.
        value: Expr,
        /// Span.
        span: Span,
    },
    /// Entity or plan `.update(patch)`.
    Update {
        /// Target entity expression or plan (encoded as [`Expr`] until lowered).
        target: Expr,
        /// Patch or `x => { … }` patch lambda.
        patch: Expr,
        /// Span.
        span: Span,
    },
    /// Entity or plan `.delete()`.
    Delete {
        /// Target.
        target: Expr,
        /// Span.
        span: Span,
    },
}

impl QueryPlan {
    /// Plan that is just `Table.all()` (no further stages).
    pub fn all(table: TableRef) -> Self {
        let span = table.span;
        Self {
            source: table,
            stages: vec![Stage::All { span }],
            span,
        }
    }

    /// Append a filter stage.
    pub fn filter(mut self, predicate: Expr, span: Span) -> Self {
        self.stages.push(Stage::Filter { predicate, span });
        self
    }

    /// Append a map stage.
    pub fn map(mut self, projection: Expr, span: Span) -> Self {
        self.stages.push(Stage::Map { projection, span });
        self
    }
}

/// Helper: build a one-parameter lambda (common in filter/map).
pub fn lambda1(param: impl Into<String>, body: Expr, span: Span) -> Lambda {
    Lambda {
        params: vec![param.into()],
        body: Box::new(body),
        span,
    }
}
