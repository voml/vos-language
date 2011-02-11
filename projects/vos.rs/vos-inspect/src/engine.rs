//! Inspect engine entry points.

use vos_ast::op::QueryPlan;
use vos_ast::{Document, Program};

use crate::rules::{check_prefer_explicit_load, prefer_explicit_load};
use crate::{InspectConfig, InspectReport};

/// Runs registered built-in inspect rules.
#[derive(Debug, Clone, Default)]
pub struct InspectEngine;

impl InspectEngine {
    /// Create an engine with the built-in rule set.
    pub fn new() -> Self {
        Self
    }

    /// Inspect a lowered query plan (preferred when the host already has IR).
    pub fn inspect_plan(
        &self,
        plan: &QueryPlan,
        document: &Document,
        config: &InspectConfig,
    ) -> InspectReport {
        let mut report = InspectReport::new();
        check_prefer_explicit_load(plan, document, config, &mut report);
        report
    }

    /// Inspect a parsed operation program (best-effort pipeline lowering).
    pub fn inspect_program(
        &self,
        program: &Program,
        document: &Document,
        config: &InspectConfig,
    ) -> InspectReport {
        let mut report = InspectReport::new();
        prefer_explicit_load::check_program(program, document, config, &mut report);
        report
    }
}
