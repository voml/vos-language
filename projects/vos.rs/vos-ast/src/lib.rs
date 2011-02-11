//! Syntax data types for **VOS — Virtual Object Schema**.
//!
//! This crate has no parser, executor, migration, or database dependency.
//! Consumers (YYDB, generators, push-mode adapters) take a versioned VOS
//! document and lower it themselves.
//!
//! **Schema AST** (`Document`, `table`, …) and **expression / operation AST**
//! ([`expr`], [`op`]) live here.

#![warn(missing_docs)]

use serde::{Deserialize, Serialize};

/// Initial field-identity catalog IR (`FieldId` + virtual slots).
pub mod catalog;
/// Stable diagnostic codes (`VOS-PROJECTION-****`, `VOS-OP-****`, …).
pub mod codes;
/// Expression / program AST (`let`, projections, …).
pub mod expr;
/// Lazy query plan + execution IR.
pub mod op;

pub use catalog::{
    CatalogSnapshot, FieldId, FieldSlot, Revisions, TypeEntry, TypeId, TypeKind, VirtualFieldIndex,
    catalog_from_document,
};
pub use expr::{
    BinaryOp, Expr, FieldInit, FnDecl, FnKind, FnParam, Lambda, Let, PathSep, Program, ProjItem,
    Stmt, UnaryOp, projection_result_names,
};
pub use op::{Exec, Patch, QueryPlan, SortDir, SortKey, Stage, TableRef, lambda1};

/// A byte range in one VOS source file (`[start, end)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    /// Inclusive byte offset of the first character.
    pub start: usize,
    /// Exclusive byte offset following the last character.
    pub end: usize,
}

impl Span {
    /// Create a span from a half-open byte range.
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Empty span at `offset`.
    pub fn empty(offset: usize) -> Self {
        Self {
            start: offset,
            end: offset,
        }
    }
}

/// Structured diagnostic with file-relative span and optional repair hint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Stable machine code when known (`VOS-PROJECTION-0003`, …).
    pub code: Option<String>,
    /// Human-readable reason.
    pub message: String,
    /// Location in the source document.
    pub span: Span,
    /// Suggested repair when known.
    pub hint: Option<String>,
}

impl Diagnostic {
    /// Build a diagnostic without a stable code.
    pub fn new(message: impl Into<String>, span: Span, hint: Option<impl Into<String>>) -> Self {
        Self {
            code: None,
            message: message.into(),
            span,
            hint: hint.map(Into::into),
        }
    }

    /// Build a diagnostic with a stable [`codes`] value.
    pub fn with_code(
        code: impl Into<String>,
        message: impl Into<String>,
        span: Span,
        hint: Option<impl Into<String>>,
    ) -> Self {
        Self {
            code: Some(code.into()),
            message: message.into(),
            span,
            hint: hint.map(Into::into),
        }
    }
}

/// One or more diagnostics from parse or semantic check.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Diagnostics {
    /// Errors in source order when available.
    pub errors: Vec<Diagnostic>,
}

impl Diagnostics {
    /// True when no errors were recorded.
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Push one error.
    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.errors.push(diagnostic);
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(code) = &self.code {
            write!(f, "{code}: ")?;
        }
        write!(
            f,
            "{} (bytes {}..{})",
            self.message, self.span.start, self.span.end
        )?;
        if let Some(hint) = &self.hint {
            write!(f, "; hint: {hint}")?;
        }
        Ok(())
    }
}

impl std::error::Error for Diagnostic {}

impl std::fmt::Display for Diagnostics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.errors.is_empty() {
            return write!(f, "no VOS diagnostics");
        }
        for (i, err) in self.errors.iter().enumerate() {
            if i > 0 {
                write!(f, "; ")?;
            }
            write!(f, "{err}")?;
        }
        Ok(())
    }
}

impl std::error::Error for Diagnostics {}

/// Path segments of a `namespace a::b::c` declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NamespacePath {
    /// Segments in order (`a`, `b`, `c`).
    pub segments: Vec<String>,
    /// Source span of the path.
    pub span: Span,
}

impl NamespacePath {
    /// Join segments with `::`.
    pub fn display(&self) -> String {
        self.segments.join("::")
    }
}

/// Field-level attribute preserved for providers and tooling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FieldAttribute {
    /// Primary key (`[primary]` / `@@name`).
    Primary,
    /// Unique key (`[unique]` / `@name`).
    Unique,
}

/// Builtin scalar names (case-sensitive).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum BuiltinType {
    /// Signed integers.
    I8,
    /// Signed 16-bit.
    I16,
    /// Signed 32-bit.
    I32,
    /// Signed 64-bit.
    I64,
    /// Unsigned 8-bit.
    U8,
    /// Unsigned 16-bit.
    U16,
    /// Unsigned 32-bit.
    U32,
    /// Unsigned 64-bit.
    U64,
    /// Binary32 float.
    F32,
    /// Binary64 float.
    F64,
    /// Boolean.
    Bool,
    /// UTF-8 text.
    Utf8,
    /// UTF-16 text.
    Utf16,
    /// UUID (lowercase name only).
    Uuid,
    /// Exact decimal (`decimal` / `d128`).
    Decimal,
    /// Calendar date without timezone (`date`, ISO-8601 `YYYY-MM-DD`).
    Date,
    /// Time of day without timezone (`time`, ISO-8601 `HH:MM:SS[.fraction]`).
    Time,
    /// UTC timestamp (`DateTime<UTC>` / `datetime`).
    DateTimeUtc,
    /// Small binary payload.
    Bytes,
}

impl BuiltinType {
    /// Parse a builtin type name; `None` means a user-defined type name.
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "i8" => Some(Self::I8),
            "i16" => Some(Self::I16),
            "i32" => Some(Self::I32),
            "i64" => Some(Self::I64),
            "u8" => Some(Self::U8),
            "u16" => Some(Self::U16),
            "u32" => Some(Self::U32),
            "u64" => Some(Self::U64),
            "f32" => Some(Self::F32),
            "f64" => Some(Self::F64),
            "bool" => Some(Self::Bool),
            "utf8" => Some(Self::Utf8),
            "utf16" => Some(Self::Utf16),
            "uuid" => Some(Self::Uuid),
            "decimal" | "d128" => Some(Self::Decimal),
            "date" => Some(Self::Date),
            "time" => Some(Self::Time),
            "datetime" => Some(Self::DateTimeUtc),
            "bytes" => Some(Self::Bytes),
            _ => None,
        }
    }

    /// Canonical VOS spelling for diagnostics.
    pub fn as_vos(self) -> &'static str {
        match self {
            Self::I8 => "i8",
            Self::I16 => "i16",
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::U8 => "u8",
            Self::U16 => "u16",
            Self::U32 => "u32",
            Self::U64 => "u64",
            Self::F32 => "f32",
            Self::F64 => "f64",
            Self::Bool => "bool",
            Self::Utf8 => "utf8",
            Self::Utf16 => "utf16",
            Self::Uuid => "uuid",
            Self::Decimal => "decimal",
            Self::Date => "date",
            Self::Time => "time",
            Self::DateTimeUtc => "DateTime<UTC>",
            Self::Bytes => "bytes",
        }
    }
}

/// Type expression after wrappers are applied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TypeExpr {
    /// Builtin scalar.
    Builtin(BuiltinType),
    /// Named user type (`table` / `class` / …).
    Named(String),
    /// Primary-key reference to `inner`.
    Reference(Box<TypeExpr>),
    /// Optional wrapper.
    Optional(Box<TypeExpr>),
    /// List wrapper.
    List(Box<TypeExpr>),
    /// Fixed-dimension embedding `vector<N>`.
    Vector {
        /// Dimension `N`.
        dim: u32,
    },
    /// Logical large file type.
    File,
}

/// Literal used as a field default.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Literal {
    /// `null`.
    Null,
    /// Boolean literal.
    Bool(bool),
    /// Integer literal text as written.
    Int(String),
    /// Float literal text as written.
    Float(String),
    /// Quoted string contents (escapes not yet expanded beyond the lexer).
    String(String),
    /// Bare identifier (for example an enum variant).
    Ident(String),
}

/// One field on a `table` (or later `class`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Field {
    /// Field name (from `name:` or from `@` / `@@` shorthand).
    pub name: String,
    /// Field type.
    pub ty: TypeExpr,
    /// Attributes such as primary / unique.
    pub attrs: Vec<FieldAttribute>,
    /// Optional default literal.
    pub default: Option<Literal>,
    /// Declaration span.
    pub span: Span,
}

impl Field {
    /// True when this field is marked primary.
    pub fn is_primary(&self) -> bool {
        self.attrs.contains(&FieldAttribute::Primary)
    }

    /// True when this field is marked unique.
    pub fn is_unique(&self) -> bool {
        self.attrs.contains(&FieldAttribute::Unique)
    }
}

/// A persistence `table` declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Table {
    /// Table name.
    pub name: String,
    /// Fields in source order (identity is still by name).
    pub fields: Vec<Field>,
    /// Declaration span.
    pub span: Span,
}

impl Table {
    /// Primary-key fields (providers currently expect exactly one).
    pub fn primary_fields(&self) -> impl Iterator<Item = &Field> {
        self.fields.iter().filter(|f| f.is_primary())
    }
}

/// A non-persistent `class` (DTO / domain record).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Class {
    /// Class name.
    pub name: String,
    /// Fields in source order (identity is still by name).
    pub fields: Vec<Field>,
    /// Declaration span.
    pub span: Span,
}

/// One variant in an `enums` or `flags` declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumVariant {
    /// Variant name.
    pub name: String,
    /// Numeric value text as written (`1`, `0x10`, …).
    pub value: String,
    /// Declaration span.
    pub span: Span,
}

/// Numeric `enums` declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Enums {
    /// Enum name.
    pub name: String,
    /// Variants in source order.
    pub variants: Vec<EnumVariant>,
    /// Declaration span.
    pub span: Span,
}

/// Bitmask `flags` declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Flags {
    /// Flags name.
    pub name: String,
    /// Variants in source order.
    pub variants: Vec<EnumVariant>,
    /// Declaration span.
    pub span: Span,
}

/// Explicit removal consumed by migration tooling.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Obsolete {
    /// `obsolete table Name;`
    Table {
        /// Removed table name.
        name: String,
        /// Declaration span.
        span: Span,
    },
    /// `obsolete field Type.field;`
    Field {
        /// Parent type name.
        parent: String,
        /// Removed field name.
        field: String,
        /// Declaration span.
        span: Span,
    },
}

/// Top-level item in a `.vos` document (subset expands over time).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Item {
    /// `table Name { … }`.
    Table(Table),
    /// `class Name { … }`.
    Class(Class),
    /// `enums Name { … }`.
    Enums(Enums),
    /// `flags Name { … }`.
    Flags(Flags),
    /// `obsolete …;`
    Obsolete(Obsolete),
    /// Durable `macro name(…) -> T { … }` (DDL catalog object).
    Macro(crate::expr::FnDecl),
}

/// Parsed VOS source file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    /// Optional `namespace a::b::c`. Absent ⇒ default / single-namespace mode.
    pub namespace: Option<NamespacePath>,
    /// Top-level items.
    pub items: Vec<Item>,
    /// Normalized source kept for span mapping (BOM stripped, `\n` newlines).
    /// Omitted from conformance `*.ast.json` (see `*.normalized.vos`).
    #[serde(skip_serializing, default)]
    pub source: String,
}

impl Document {
    /// Iterate tables in the document.
    pub fn tables(&self) -> impl Iterator<Item = &Table> {
        self.items.iter().filter_map(|item| match item {
            Item::Table(table) => Some(table),
            _ => None,
        })
    }

    /// Iterate classes in the document.
    pub fn classes(&self) -> impl Iterator<Item = &Class> {
        self.items.iter().filter_map(|item| match item {
            Item::Class(class) => Some(class),
            _ => None,
        })
    }
}
