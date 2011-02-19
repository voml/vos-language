//! Shared VOS inspect engine (YYDB / YYDS / CLI).
//!
//! Hard semantic diagnostics remain in parse / check / executors
//! (`VOS-OP-*`, `VOS-EXPR-*`, …). This crate owns optional, level-gated
//! `VOS-INSPECT-*` deeper checks (optional, level-gated).

#![deny(missing_docs)]

mod assoc;
mod config;
mod engine;
mod finding;
mod id;
mod level;
mod rules;

pub use config::InspectConfig;
pub use engine::InspectEngine;
pub use finding::{InspectFinding, InspectReport};
pub use id::{InspectId, codes};
pub use level::InspectLevel;

/// Re-export association helpers for hosts that share inference with inspect.
pub mod association {
    pub use crate::assoc::{
        collect_assoc_uses, load_selector_field, reference_field_names, reference_target_table,
    };
}
