use std::collections::BTreeSet;

use core::range::Range;

use serde::{Deserialize, Serialize};

use crate::ids::{OperationId, canonical_digest};
use crate::projection::{deserialize_span, serialize_span};
use crate::{ArtifactError, OperationKind, ResolvedCanonicalType};

/// Version of the canonical operation contract fingerprint derivation.
pub const CONTRACT_FINGERPRINT_VERSION: &str = "vos-contract-fingerprint-v0";

/// Effect an operation has on stored state.
///
/// The effect is declared next to the operation contract instead of being
/// inferred from the body, because VOS never lowers bodies in the contract
/// layer and a guessed effect would be a silent permission grant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OperationEffect {
    /// No stored state is read or written.
    Pure,
    /// Stored state is read but not written.
    Read,
    /// Stored state is written.
    Write,
}

/// Capability an operation requires from the execution backend.
///
/// The vocabulary is closed so that Iris and Hermes can reject an operation
/// whose backend cannot satisfy it instead of discovering it at run time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SemanticCapability {
    /// Aggregate evaluation.
    Aggregation,
    /// Text search index.
    Search,
    /// Persistent row storage.
    Storage,
    /// Vector index.
    Vector,
}

/// Result cardinality of an operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OperationCardinality {
    /// Exactly one value, or unit when the operation has no result.
    One,
    /// Zero or more values.
    Many,
}

/// One named parameter of an operation contract.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OperationParameterContract {
    /// Parameter name.
    pub name: String,
    /// Canonical parameter type.
    pub canonical_type: ResolvedCanonicalType,
}

/// Durable contract of a single VOS operation.
///
/// The contract carries identity, signature, effect, capability requirements
/// and provenance. It never carries a body expression, so no consumer can
/// re-parse VOS source from it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OperationContract {
    /// Durable operation identity.
    pub operation_id: OperationId,
    /// Declaration kind.
    pub kind: OperationKind,
    /// Canonical operation name.
    pub name: String,
    /// Canonical namespace-qualified path ending with the operation name.
    pub canonical_path: Vec<String>,
    /// Named parameter tuple in declaration order.
    pub parameters: Vec<OperationParameterContract>,
    /// Canonical result type, absent when the operation has no result.
    pub result_type: Option<ResolvedCanonicalType>,
    /// Result cardinality derived from the declared result type.
    pub cardinality: OperationCardinality,
    /// Declared storage effect.
    pub effect: OperationEffect,
    /// Required backend capabilities in canonical order.
    pub capabilities: Vec<SemanticCapability>,
    /// Source unit the operation was projected from.
    pub source_unit_id: u32,
    /// Declaration span inside the source unit.
    #[serde(
        serialize_with = "serialize_span",
        deserialize_with = "deserialize_span"
    )]
    pub span: Range<usize>,
}

impl OperationContract {
    /// Cardinality implied by a declared result type.
    pub fn cardinality_of(result_type: &Option<ResolvedCanonicalType>) -> OperationCardinality {
        match result_type {
            Some(ResolvedCanonicalType::List(_)) => OperationCardinality::Many,
            _ => OperationCardinality::One,
        }
    }

    /// Rejects a contract that cannot be trusted by a downstream consumer.
    pub fn validate(&self) -> Result<(), ArtifactError> {
        OperationId::parse(self.operation_id.as_str()).map_err(|_| ArtifactError {
            code: "OPC010".to_owned(),
            message: "operation contract carries a malformed operation id".to_owned(),
        })?;
        if self.name.trim().is_empty()
            || self.canonical_path.is_empty()
            || self.canonical_path.last() != Some(&self.name)
        {
            return Err(ArtifactError {
                code: "OPC011".to_owned(),
                message: "operation canonical path must end with its own name".to_owned(),
            });
        }
        let mut names = BTreeSet::new();
        for parameter in &self.parameters {
            if parameter.name.trim().is_empty() {
                return Err(ArtifactError {
                    code: "OPC012".to_owned(),
                    message: "operation parameter names must not be empty".to_owned(),
                });
            }
            if !names.insert(parameter.name.as_str()) {
                return Err(ArtifactError {
                    code: "OPC012".to_owned(),
                    message: "operation parameter names must be unique".to_owned(),
                });
            }
        }
        if self.cardinality != Self::cardinality_of(&self.result_type) {
            return Err(ArtifactError {
                code: "OPC013".to_owned(),
                message: "operation cardinality disagrees with its result type".to_owned(),
            });
        }
        let mut capabilities = self.capabilities.iter().copied();
        let mut previous = None;
        for capability in capabilities.by_ref() {
            if previous.is_some_and(|value| value >= capability) {
                return Err(ArtifactError {
                    code: "OPC014".to_owned(),
                    message: "operation capabilities must be sorted and unique".to_owned(),
                });
            }
            previous = Some(capability);
        }
        Ok(())
    }
}

/// Canonical fingerprint of a set of operation contracts.
///
/// Operations are ordered by durable identity, and source units, spans and
/// declaration order are excluded, so the fingerprint is stable across source
/// units, whitespace, comments and build order while still changing on any
/// rename, retype or effect change.
pub fn operation_contract_fingerprint(operations: &[OperationContract]) -> String {
    let mut views = operations.iter().collect::<Vec<_>>();
    views.sort_by(|left, right| left.operation_id.cmp(&right.operation_id));
    let views = views
        .into_iter()
        .map(|operation| CanonicalOperationContract {
            fingerprint_version: CONTRACT_FINGERPRINT_VERSION,
            operation_id: &operation.operation_id,
            kind: operation.kind,
            canonical_path: &operation.canonical_path,
            parameters: operation
                .parameters
                .iter()
                .map(|parameter| CanonicalParameter {
                    name: &parameter.name,
                    canonical_type: &parameter.canonical_type,
                })
                .collect(),
            result_type: &operation.result_type,
            cardinality: operation.cardinality,
            effect: operation.effect,
            capabilities: sorted_capabilities(&operation.capabilities),
        })
        .collect::<Vec<_>>();
    canonical_digest(&views)
}

fn sorted_capabilities(capabilities: &[SemanticCapability]) -> Vec<SemanticCapability> {
    let mut capabilities = capabilities.to_vec();
    capabilities.sort_unstable();
    capabilities.dedup();
    capabilities
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CanonicalOperationContract<'a> {
    fingerprint_version: &'static str,
    operation_id: &'a OperationId,
    kind: OperationKind,
    canonical_path: &'a [String],
    parameters: Vec<CanonicalParameter<'a>>,
    result_type: &'a Option<ResolvedCanonicalType>,
    cardinality: OperationCardinality,
    effect: OperationEffect,
    capabilities: Vec<SemanticCapability>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CanonicalParameter<'a> {
    name: &'a str,
    canonical_type: &'a ResolvedCanonicalType,
}
