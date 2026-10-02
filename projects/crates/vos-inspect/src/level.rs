//! Per-rule rigor levels.

use serde::{Deserialize, Serialize};

/// How strictly a single inspect rule is enforced.
///
/// Hosts map [`Deny`](Self::Deny) to hard failure (exit code / CI) and
/// [`Warn`](Self::Warn) to non-fatal diagnostics. [`Allow`](Self::Allow) and
/// [`Off`](Self::Off) both suppress emission; `Off` is an explicit disable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum InspectLevel {
    /// Do not run or emit this rule.
    Off,
    /// Run for analysis hooks if needed, but do not emit findings (default for
    /// most style / policy rules).
    #[default]
    Allow,
    /// Emit a non-fatal finding.
    Warn,
    /// Emit a finding that hosts should treat as failure.
    Deny,
}

impl InspectLevel {
    /// Parse a level name (`off` / `allow` / `warn` / `deny`), case-insensitive.
    pub fn parse(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "off" => Some(Self::Off),
            "allow" => Some(Self::Allow),
            "warn" => Some(Self::Warn),
            "deny" => Some(Self::Deny),
            _ => None,
        }
    }

    /// Whether this level produces a user-visible finding.
    pub const fn emits(self) -> bool {
        matches!(self, Self::Warn | Self::Deny)
    }

    /// Whether hosts should fail the run when this level is resolved.
    pub const fn is_deny(self) -> bool {
        matches!(self, Self::Deny)
    }

    /// Stable lowercase name for CLI / logs (`warn`, `deny`, …).
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Allow => "allow",
            Self::Warn => "warn",
            Self::Deny => "deny",
        }
    }
}
