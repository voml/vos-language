//! Stable inspect rule identifiers and `VOS-INSPECT-*` codes.

use serde::{Deserialize, Serialize};

/// Built-in inspect rule identity.
///
/// New rules get a new variant + code. Unknown codes from config files are
/// ignored (or reported by the host), never mapped onto semantic `VOS-OP-*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum InspectId {
    /// Prefer an explicit `.load` when association fields are used via inference.
    ///
    /// Default level: [`Allow`](crate::InspectLevel::Allow) — association
    /// use-site inference is the language default; this rule is optional rigor.
    PreferExplicitLoad,
}

impl InspectId {
    /// Stable diagnostic code (`VOS-INSPECT-…`).
    pub const fn code(self) -> &'static str {
        match self {
            Self::PreferExplicitLoad => codes::PREFER_EXPLICIT_LOAD,
        }
    }

    /// Short kebab-case name for config maps (`prefer-explicit-load`).
    pub const fn name(self) -> &'static str {
        match self {
            Self::PreferExplicitLoad => "prefer-explicit-load",
        }
    }

    /// All built-in rule ids.
    pub const fn all() -> &'static [InspectId] {
        &[Self::PreferExplicitLoad]
    }

    /// Resolve from code (`VOS-INSPECT-LOAD-001`) or kebab name.
    pub fn parse(key: &str) -> Option<Self> {
        let key = key.trim();
        for id in Self::all() {
            if id.code().eq_ignore_ascii_case(key) || id.name().eq_ignore_ascii_case(key) {
                return Some(*id);
            }
        }
        None
    }

    /// Documented default rigor when config does not override.
    pub const fn default_level(self) -> crate::InspectLevel {
        match self {
            Self::PreferExplicitLoad => crate::InspectLevel::Allow,
        }
    }
}

/// String constants for inspect codes.
pub mod codes {
    /// Prefer explicit `.load` when association use sites are inferred.
    pub const PREFER_EXPLICIT_LOAD: &str = "VOS-INSPECT-LOAD-001";
}
