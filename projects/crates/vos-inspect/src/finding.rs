//! Inspect findings and reports.

use serde::{Deserialize, Serialize};
use vos_ast::{Diagnostic, Span};

use crate::{InspectId, InspectLevel};

/// One leveled inspect finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectFinding {
    /// Which rule fired.
    pub id: InspectId,
    /// Level after config resolution (`Warn` or `Deny` when emitted).
    pub level: InspectLevel,
    /// Human-readable message.
    pub message: String,
    /// Source span when known.
    pub span: Span,
    /// Optional fix / style hint.
    pub hint: Option<String>,
}

impl InspectFinding {
    /// Build a finding (caller must only emit for levels that [`InspectLevel::emits`]).
    pub fn new(
        id: InspectId,
        level: InspectLevel,
        message: impl Into<String>,
        span: Span,
        hint: Option<String>,
    ) -> Self {
        Self {
            id,
            level,
            message: message.into(),
            span,
            hint,
        }
    }

    /// Map into a language [`Diagnostic`] (code = inspect code).
    pub fn to_diagnostic(&self) -> Diagnostic {
        Diagnostic {
            code: Some(self.id.code().to_owned()),
            message: self.message.clone(),
            span: self.span,
            hint: self.hint.clone(),
        }
    }
}

/// Aggregate inspect results for one check pass.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectReport {
    /// Findings that passed the level gate (`warn` / `deny` only).
    pub findings: Vec<InspectFinding>,
}

impl InspectReport {
    /// Empty report.
    pub fn new() -> Self {
        Self::default()
    }

    /// Push a finding if `level` emits.
    pub fn push(&mut self, finding: InspectFinding) {
        if finding.level.emits() {
            self.findings.push(finding);
        }
    }

    /// Findings at deny level.
    pub fn denials(&self) -> impl Iterator<Item = &InspectFinding> {
        self.findings.iter().filter(|f| f.level.is_deny())
    }

    /// Findings at warn level.
    pub fn warnings(&self) -> impl Iterator<Item = &InspectFinding> {
        self.findings
            .iter()
            .filter(|f| matches!(f.level, InspectLevel::Warn))
    }

    /// True when no deny-level findings (CI-friendly).
    pub fn passed(&self) -> bool {
        self.denials().next().is_none()
    }

    /// Diagnostics for deny findings (hosts may merge into hard errors).
    pub fn denial_diagnostics(&self) -> Vec<Diagnostic> {
        self.denials().map(InspectFinding::to_diagnostic).collect()
    }

    /// Diagnostics for warn findings.
    pub fn warning_diagnostics(&self) -> Vec<Diagnostic> {
        self.warnings().map(InspectFinding::to_diagnostic).collect()
    }
}
