//! Per-rule inspect configuration.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{InspectId, InspectLevel};

/// Inspect configuration shared by YYDB / YYDS / CLI hosts.
///
/// Resolution order for a rule:
/// 1. Explicit override in [`by_rule`](Self::by_rule) (by [`InspectId::code`] or
///    [`InspectId::name`])
/// 2. That rule's [`InspectId::default_level`]
/// 3. Never the global [`default_level`](Self::default_level) alone — the global
///    default only applies when looking up **unknown** keys or when using
///    [`level_or_default`](Self::level_or_default) helpers for host-defined
///    extensions. Built-in rules always have a documented per-rule default.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectConfig {
    /// Fallback used by hosts for unknown / extension rule keys.
    #[serde(default)]
    pub default_level: InspectLevel,
    /// Overrides keyed by rule code (`VOS-INSPECT-LOAD-001`) or kebab name
    /// (`prefer-explicit-load`).
    #[serde(default)]
    pub by_rule: BTreeMap<String, InspectLevel>,
}

impl Default for InspectConfig {
    fn default() -> Self {
        Self::permissive()
    }
}

impl InspectConfig {
    /// All built-in rules at their documented defaults (most style rules `allow`).
    pub fn permissive() -> Self {
        Self {
            default_level: InspectLevel::Allow,
            by_rule: BTreeMap::new(),
        }
    }

    /// Treat every built-in rule as [`InspectLevel::Warn`] unless overridden.
    pub fn pedantic() -> Self {
        let mut by_rule = BTreeMap::new();
        for id in InspectId::all() {
            by_rule.insert(id.name().to_owned(), InspectLevel::Warn);
        }
        Self {
            default_level: InspectLevel::Warn,
            by_rule,
        }
    }

    /// Set rigor for one built-in rule (stores under the kebab name).
    pub fn set(&mut self, id: InspectId, level: InspectLevel) -> &mut Self {
        self.by_rule.insert(id.name().to_owned(), level);
        self
    }

    /// Resolved level for a built-in rule.
    pub fn level(&self, id: InspectId) -> InspectLevel {
        if let Some(level) = self.lookup_override(id) {
            return level;
        }
        id.default_level()
    }

    /// Resolve an arbitrary key (code or name); unknown keys use `default_level`.
    pub fn level_or_default(&self, key: &str) -> InspectLevel {
        if let Some(id) = InspectId::parse(key) {
            return self.level(id);
        }
        self.by_rule.get(key).copied().unwrap_or(self.default_level)
    }

    fn lookup_override(&self, id: InspectId) -> Option<InspectLevel> {
        self.by_rule
            .get(id.name())
            .or_else(|| self.by_rule.get(id.code()))
            .copied()
    }
}
