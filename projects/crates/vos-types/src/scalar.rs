use core::fmt;

use serde::{Deserialize, Serialize};

/// Canonical scalar type of a VOS contract.
///
/// VOS does not define storage semantics of its own. Every scalar below is the
/// exact VOS spelling that the storage layer already owns, so a contract never
/// needs a private storage type and no value is re-interpreted on the way to
/// YYDB. The alignment is:
///
/// | VOS scalar | YYDB scalar | note |
/// |---|---|---|
/// | `i8` `i16` `i32` `i64` | same integer kind | width is part of the contract |
/// | `u8` `u16` `u32` `u64` | same integer kind | unsigned values never widen into a signed kind |
/// | `f32` `f64` | same float kind | precision is part of the contract |
/// | `bool` | `bool` | no truthiness conversion |
/// | `utf8` | `utf8` | text is UTF-8 |
/// | `utf16` | `utf16` | kept distinct from `utf8` and never merged into one host string |
/// | `uuid` | `uuid` | lowercase canonical spelling |
/// | `decimal` | `d128` | `d128` is a VOS source alias normalized to `decimal` |
/// | `date` | `date` | calendar date without time |
/// | `time` | `time` | wall-clock time without date |
/// | `datetime` | `DateTime<UTC>` | always UTC |
/// | `bytes` | binary | no host-specific text encoding |
///
/// `vector<N>`, `file` and `stream<T>` are not scalars and stay in
/// [`crate::CanonicalType`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CanonicalScalar {
    /// Signed 8-bit integer.
    I8,
    /// Signed 16-bit integer.
    I16,
    /// Signed 32-bit integer.
    I32,
    /// Signed 64-bit integer.
    I64,
    /// Unsigned 8-bit integer.
    U8,
    /// Unsigned 16-bit integer.
    U16,
    /// Unsigned 32-bit integer.
    U32,
    /// Unsigned 64-bit integer.
    U64,
    /// 32-bit IEEE-754 float.
    F32,
    /// 64-bit IEEE-754 float.
    F64,
    /// Boolean.
    Bool,
    /// UTF-8 text.
    Utf8,
    /// UTF-16 text.
    Utf16,
    /// Canonical lowercase UUID.
    Uuid,
    /// Exact decimal number.
    Decimal,
    /// Calendar date.
    Date,
    /// Wall-clock time.
    Time,
    /// UTC date and time.
    DateTime,
    /// Binary payload.
    Bytes,
}

impl CanonicalScalar {
    /// Every canonical scalar in declaration order.
    pub const ALL: [Self; 19] = [
        Self::I8,
        Self::I16,
        Self::I32,
        Self::I64,
        Self::U8,
        Self::U16,
        Self::U32,
        Self::U64,
        Self::F32,
        Self::F64,
        Self::Bool,
        Self::Utf8,
        Self::Utf16,
        Self::Uuid,
        Self::Decimal,
        Self::Date,
        Self::Time,
        Self::DateTime,
        Self::Bytes,
    ];

    /// Reads a canonical scalar by its exact VOS spelling.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|scalar| scalar.as_str() == name)
    }

    /// Reads a scalar from a VOS source type name.
    ///
    /// Source aliases are normalized here so that every later stage only sees
    /// the canonical spelling. `d128` is the only alias today and normalizes
    /// to `decimal`.
    pub fn from_source_name(name: &str) -> Option<Self> {
        match name {
            "d128" => Some(Self::Decimal),
            other => Self::from_name(other),
        }
    }

    /// Returns the canonical VOS spelling of this scalar.
    pub const fn as_str(self) -> &'static str {
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
            Self::DateTime => "datetime",
            Self::Bytes => "bytes",
        }
    }
}

impl fmt::Display for CanonicalScalar {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}
