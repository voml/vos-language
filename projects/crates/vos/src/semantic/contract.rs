//! Authored operation semantics and [`OperationContract`] construction.

use std::collections::{BTreeMap, BTreeSet};

use core::range::Range;

use vos_types::{
    ArtifactError, ContractDiagnostic, OperationContract, OperationEffect, OperationId,
    OperationParameterContract, OperationProjection, ResolvedTypeContract, SemanticCapability,
};

use super::identity::resolve_type;

/// Declared effect and capability requirements of one operation.
///
/// Effects and capabilities are authored explicitly because the VOS contract
/// layer never lowers a body expression. An operation without a declaration is
/// rejected instead of being assumed pure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationSemantics {
    /// Durable identity of the operation these semantics belong to.
    pub operation_id: OperationId,
    /// Declared storage effect.
    pub effect: OperationEffect,
    /// Required backend capabilities, stored in canonical order.
    pub capabilities: Vec<SemanticCapability>,
}

impl OperationSemantics {
    /// Builds a declaration with sorted, unique capabilities.
    pub fn new(
        operation_id: OperationId,
        effect: OperationEffect,
        mut capabilities: Vec<SemanticCapability>,
    ) -> Self {
        capabilities.sort_unstable();
        capabilities.dedup();
        Self {
            operation_id,
            effect,
            capabilities,
        }
    }
}

/// Builds validated operation contracts from Oak projections.
///
/// Operations are read from every source unit so provenance survives, while
/// identity, effect and capability requirements come from `semantics`. The
/// result is ordered by durable identity, which makes the contract independent
/// of source-unit and declaration order.
pub fn build_operation_contracts(
    units: &[(u32, &[OperationProjection])],
    semantics: &[OperationSemantics],
    types: &[ResolvedTypeContract],
) -> Result<Vec<OperationContract>, Vec<ContractDiagnostic>> {
    let type_ids = types
        .iter()
        .map(|item| (item.canonical_path.clone(), item.type_id))
        .collect::<BTreeMap<_, _>>();
    let mut diagnostics = Vec::new();
    let mut declared = BTreeMap::new();
    for entry in semantics {
        if declared.insert(entry.operation_id.clone(), entry).is_some() {
            diagnostics.push(diagnostic(
                "OPC002",
                &format!(
                    "operation semantics declared twice for {}",
                    entry.operation_id
                ),
            ));
        }
    }

    let mut seen = BTreeSet::new();
    let mut used = BTreeSet::new();
    let mut operations = Vec::new();
    for (source_unit_id, unit_operations) in units {
        for operation in *unit_operations {
            if !seen.insert(operation.operation_id.clone()) {
                diagnostics.push(diagnostic(
                    "OPC003",
                    &format!(
                        "operation {} is declared in more than one unit",
                        operation.operation_id
                    ),
                ));
                continue;
            }
            let Some(entry) = declared.get(&operation.operation_id) else {
                diagnostics.push(diagnostic(
                    "OPC001",
                    &format!(
                        "operation {} has no declared effect",
                        operation.operation_id
                    ),
                ));
                continue;
            };
            used.insert(operation.operation_id.clone());
            let namespace = &operation.canonical_path[..operation.canonical_path.len() - 1];
            let Some(parameters) = operation
                .parameters
                .iter()
                .map(|parameter| {
                    resolve_type(&parameter.type_expr, namespace, &type_ids, &mut diagnostics).map(
                        |canonical_type| OperationParameterContract {
                            name: parameter.name.clone(),
                            canonical_type,
                        },
                    )
                })
                .collect::<Option<Vec<_>>>()
            else {
                continue;
            };
            let result_type = match operation.return_type.as_ref() {
                Some(result) => {
                    match resolve_type(result, namespace, &type_ids, &mut diagnostics) {
                        Some(result) => Some(result),
                        None => continue,
                    }
                }
                None => None,
            };
            let contract = OperationContract {
                operation_id: operation.operation_id.clone(),
                kind: operation.kind,
                name: operation.name.clone(),
                canonical_path: operation.canonical_path.clone(),
                parameters,
                cardinality: OperationContract::cardinality_of(&result_type),
                result_type,
                effect: entry.effect,
                capabilities: entry.capabilities.clone(),
                source_unit_id: *source_unit_id,
                span: operation.span,
            };
            match contract.validate() {
                Ok(()) => operations.push(contract),
                Err(error) => diagnostics.push(contract_diagnostic(&error, Some(operation.span))),
            }
        }
    }

    for operation_id in declared.keys() {
        if !used.contains(operation_id) {
            diagnostics.push(diagnostic(
                "OPC004",
                &format!("operation semantics target unknown operation {operation_id}"),
            ));
        }
    }

    if diagnostics.is_empty() {
        operations.sort_by(|left, right| left.operation_id.cmp(&right.operation_id));
        Ok(operations)
    } else {
        Err(diagnostics)
    }
}

fn diagnostic(code: &str, message: &str) -> ContractDiagnostic {
    ContractDiagnostic {
        code: code.to_owned(),
        message: message.to_owned(),
        span: None,
    }
}

fn contract_diagnostic(error: &ArtifactError, span: Option<Range<usize>>) -> ContractDiagnostic {
    ContractDiagnostic {
        code: error.code.clone(),
        message: error.message.clone(),
        span,
    }
}
