//! Authored operation semantics and [`OperationContract`] construction.

use std::collections::{BTreeMap, BTreeSet};

use core::range::Range;

use vos_types::{
    ArtifactError, AttributeContract, CanonicalType, ContractAttribute, ContractDiagnostic,
    MethodId, MethodIdentityKey, MethodStreamMode, OperationContract, OperationEffect, OperationId,
    OperationParameterContract, OperationProjection, ResolvedTypeContract, SemanticCapability,
    ServiceContract, ServiceId, ServiceMethodContract, ServiceProjection, canonical_type_key,
};

use crate::registry::{IdentityRegistry, method_key, operation_key, service_key};

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
///
/// When `registry` is `Some`, each operation is pre-allocated a monotonic,
/// persistent `stable_id_hint` keyed on its canonical path. `None` preserves
/// the previous behaviour (all hints stay unset).
pub fn build_operation_contracts(
    units: &[(u32, &[OperationProjection])],
    semantics: &[OperationSemantics],
    types: &[ResolvedTypeContract],
    mut registry: Option<&mut IdentityRegistry>,
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
                stable_id_hint: registry
                    .as_mut()
                    .map(|r| r.ensure_hint(&operation_key(&operation.canonical_path))),
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

/// Authored declaration of one service method.
///
/// Request and response types are authored as canonical VOS types and must
/// agree with the Oak-projected method body, so the declaration can never
/// invent a signature the source does not have. Types are resolved against the
/// declared type contracts, so a method can never reference a type that the
/// contract does not define.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServiceMethodDecl {
    /// Method name inside the owning service.
    pub name: String,
    /// Request type of the method.
    pub request_type: CanonicalType,
    /// Response type of the method. For `server-stream` this is the element type.
    pub response_type: CanonicalType,
    /// Invocation mode.
    pub stream: MethodStreamMode,
    /// Explicit binding to the operation that implements this method.
    pub operation_id: OperationId,
}

/// Authored contract declaration of one service.
///
/// The service identity and the method signatures come from the Oak projection.
/// The declaration adds only the explicit operation binding and is rejected when
/// it disagrees with the projected method names, request parameters, return
/// types or stream markers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServiceContractDecl {
    /// Durable identity of the projected service this declaration belongs to.
    pub service_id: ServiceId,
    /// Declared methods.
    pub methods: Vec<ServiceMethodDecl>,
}

/// Builds validated service contracts from Oak projections and declarations.
///
/// The source projection owns the method names, the single named request
/// parameter, the return type and the `stream` marker, while the declaration
/// owns the operation binding. A declaration that disagrees with the source is
/// rejected instead of being silently rewritten.
///
/// Diagnostics: `SVC001` projected service without a declaration, `SVC002`
/// duplicate declaration, `SVC003` declaration for an unknown service, `SVC004`
/// method binding an operation that has no contract, `SVC005` operation bound
/// by two services, `SVC006` the same service projected from two units,
/// `SVC007` malformed service attributes, `SVC008` malformed method attributes,
/// `SVC018` declared method missing from the source, `SVC019` source method
/// without a declaration, `SVC020` source method without exactly one request
/// parameter, `SVC021` request type mismatch, `SVC022` response type mismatch,
/// `SVC023` stream marker mismatch, `SVC024` source method without a return type.
pub fn build_service_contracts(
    units: &[(u32, &[ServiceProjection])],
    declarations: &[ServiceContractDecl],
    operations: &[OperationContract],
    types: &[ResolvedTypeContract],
    mut registry: Option<&mut IdentityRegistry>,
) -> Result<Vec<ServiceContract>, Vec<ContractDiagnostic>> {
    let type_ids = types
        .iter()
        .map(|item| (item.canonical_path.clone(), item.type_id))
        .collect::<BTreeMap<_, _>>();
    let operation_ids = operations
        .iter()
        .map(|operation| operation.operation_id.clone())
        .collect::<BTreeSet<_>>();
    let mut diagnostics = Vec::new();
    let mut declared = BTreeMap::new();
    for declaration in declarations {
        if declared
            .insert(declaration.service_id.clone(), declaration)
            .is_some()
        {
            diagnostics.push(diagnostic(
                "SVC002",
                &format!(
                    "service declaration repeated for {}",
                    declaration.service_id
                ),
            ));
        }
    }

    let mut seen = BTreeSet::new();
    let mut used = BTreeSet::new();
    let mut bound = BTreeMap::new();
    let mut services = Vec::new();
    for (source_unit_id, unit_services) in units {
        for service in *unit_services {
            if !seen.insert(service.service_id.clone()) {
                diagnostics.push(diagnostic(
                    "SVC006",
                    &format!(
                        "service {} is declared in more than one unit",
                        service.service_id
                    ),
                ));
                continue;
            }
            let Some(declaration) = declared.get(&service.service_id) else {
                diagnostics.push(diagnostic(
                    "SVC001",
                    &format!("service {} has no contract declaration", service.service_id),
                ));
                continue;
            };
            used.insert(service.service_id.clone());
            let namespace = &service.canonical_path[..service.canonical_path.len() - 1];
            let projected_methods = service
                .methods
                .iter()
                .map(|method| (method.name.as_str(), method))
                .collect::<BTreeMap<_, _>>();
            let Some(methods) = declaration
                .methods
                .iter()
                .map(|method| {
                    let Some(projected) = projected_methods.get(method.name.as_str()) else {
                        diagnostics.push(diagnostic(
                            "SVC018",
                            &format!(
                                "service method {} is missing from the source declaration",
                                method.name
                            ),
                        ));
                        return None;
                    };
                    if projected.parameters.len() != 1 {
                        diagnostics.push(diagnostic(
                            "SVC020",
                            &format!(
                                "service method {} must declare exactly one request parameter in source",
                                method.name
                            ),
                        ));
                        return None;
                    }
                    let parameter = &projected.parameters[0];
                    let request_type =
                        resolve_type(&method.request_type, namespace, &type_ids, &mut diagnostics)?;
                    let response_type = resolve_type(
                        &method.response_type,
                        namespace,
                        &type_ids,
                        &mut diagnostics,
                    )?;
                    if canonical_type_key(&parameter.type_expr)
                        != canonical_type_key(&method.request_type)
                    {
                        diagnostics.push(diagnostic(
                            "SVC021",
                            &format!(
                                "service method {} request type does not match the source parameter",
                                method.name
                            ),
                        ));
                        return None;
                    }
                    let Some(projected_response) = projected.return_type.as_ref() else {
                        diagnostics.push(diagnostic(
                            "SVC024",
                            &format!(
                                "service method {} declares no return type in source",
                                method.name
                            ),
                        ));
                        return None;
                    };
                    if canonical_type_key(projected_response)
                        != canonical_type_key(&method.response_type)
                    {
                        diagnostics.push(diagnostic(
                            "SVC022",
                            &format!(
                                "service method {} response type does not match the source return type",
                                method.name
                            ),
                        ));
                        return None;
                    }
                    let projected_stream = if projected.stream {
                        MethodStreamMode::ServerStream
                    } else {
                        MethodStreamMode::Unary
                    };
                    if projected_stream != method.stream {
                        diagnostics.push(diagnostic(
                            "SVC023",
                            &format!(
                                "service method {} stream marker does not match the source return type",
                                method.name
                            ),
                        ));
                        return None;
                    }
                    if !operation_ids.contains(&method.operation_id) {
                        diagnostics.push(diagnostic(
                            "SVC004",
                            &format!(
                                "service method {} binds unknown operation {}",
                                method.name, method.operation_id
                            ),
                        ));
                        return None;
                    }
                    if let Some(owner) =
                        bound.insert(method.operation_id.clone(), service.service_id.clone())
                        && owner != service.service_id
                    {
                        diagnostics.push(diagnostic(
                            "SVC005",
                            &format!(
                                "operation {} is bound by more than one service",
                                method.operation_id
                            ),
                        ));
                        return None;
                    }
                    let Ok(method_identity) =
                        MethodIdentityKey::new(service.service_id.clone(), &method.name)
                    else {
                        diagnostics.push(diagnostic(
                            "SVC014",
                            "service method names must be non-empty without whitespace",
                        ));
                        return None;
                    };
                    Some(ServiceMethodContract {
                        method_id: MethodId::derive(&method_identity),
                        name: method.name.clone(),
                        request_type,
                        response_type,
                        stream: method.stream,
                        operation_id: method.operation_id.clone(),
                        attributes: contract_attributes(&projected.attributes),
                        stable_id_hint: registry.as_mut().map(|r| {
                            r.ensure_hint(&method_key(&service.canonical_path, &method.name))
                        }),
                    })
                })
                .collect::<Option<Vec<_>>>()
            else {
                continue;
            };
            for name in projected_methods.keys() {
                if !declaration
                    .methods
                    .iter()
                    .any(|method| method.name == *name)
                {
                    diagnostics.push(diagnostic(
                        "SVC019",
                        &format!("source method {name} has no contract declaration"),
                    ));
                }
            }
            let contract = ServiceContract {
                service_id: service.service_id.clone(),
                name: service.name.clone(),
                canonical_path: service.canonical_path.clone(),
                attributes: contract_attributes(&service.attributes),
                methods,
                source_unit_id: *source_unit_id,
                span: service.span,
                stable_id_hint: registry
                    .as_mut()
                    .map(|r| r.ensure_hint(&service_key(&service.canonical_path))),
            };
            match contract.validate() {
                Ok(()) => services.push(contract),
                Err(error) => diagnostics.push(contract_diagnostic(&error, Some(service.span))),
            }
        }
    }

    for service_id in declared.keys() {
        if !used.contains(service_id) {
            diagnostics.push(diagnostic(
                "SVC003",
                &format!("service declaration targets unknown service {service_id}"),
            ));
        }
    }

    if diagnostics.is_empty() {
        services.sort_by(|left, right| left.service_id.cmp(&right.service_id));
        Ok(services)
    } else {
        Err(diagnostics)
    }
}

fn contract_attributes(attributes: &[AttributeContract]) -> Vec<ContractAttribute> {
    attributes
        .iter()
        .map(|attribute| ContractAttribute {
            name: attribute.name.clone(),
            syntax: attribute.syntax.clone(),
        })
        .collect()
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
