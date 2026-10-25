use std::collections::BTreeSet;

use core::range::Range;

use serde::{Deserialize, Serialize};

use crate::ids::{
    MethodId, MethodIdentityKey, OperationId, ServiceId, ServiceIdentityKey, canonical_digest,
};
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

/// Invocation mode of one service method.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MethodStreamMode {
    /// One request, one response.
    Unary,
    /// One request, a stream of responses.
    ServerStream,
}

/// One attribute carried by a service or service method contract.
///
/// Attributes are an open collection: a name VOS does not understand is kept
/// verbatim instead of being rejected. The contract carries no span, because an
/// attribute has to survive being moved between source units and hosts, and a
/// source position would make the artifact unusable outside its origin.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContractAttribute {
    /// Canonical attribute name, or the verbatim head when VOS does not know it.
    pub name: String,
    /// Original attribute syntax, preserved for round-tripping.
    pub syntax: String,
}

/// One method of a service contract.
///
/// The method name and the owning service produce the durable [`MethodId`],
/// while `operation_id` is the explicit binding to an [`OperationContract`].
/// A method never re-declares operation semantics, and its attributes never
/// change that signature.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ServiceMethodContract {
    /// Durable method identity.
    pub method_id: MethodId,
    /// Method name inside the service.
    pub name: String,
    /// Canonical request type.
    pub request_type: ResolvedCanonicalType,
    /// Canonical response type. For `server-stream` this is the element type.
    pub response_type: ResolvedCanonicalType,
    /// Invocation mode.
    pub stream: MethodStreamMode,
    /// Explicit binding to the operation that implements this method.
    pub operation_id: OperationId,
    /// Attributes declared on the method, in source order.
    pub attributes: Vec<ContractAttribute>,
}

/// Durable contract of a VOS service.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ServiceContract {
    /// Durable service identity.
    pub service_id: ServiceId,
    /// Service name.
    pub name: String,
    /// Canonical namespace-qualified path ending with the service name.
    pub canonical_path: Vec<String>,
    /// Attributes declared on the service, in source order.
    pub attributes: Vec<ContractAttribute>,
    /// Methods in canonical order.
    pub methods: Vec<ServiceMethodContract>,
    /// Source unit the service was projected from.
    pub source_unit_id: u32,
    /// Declaration span inside the source unit.
    #[serde(
        serialize_with = "serialize_span",
        deserialize_with = "deserialize_span"
    )]
    pub span: Range<usize>,
}

impl ServiceContract {
    /// Rejects a service contract that a downstream consumer cannot bind.
    pub fn validate(&self) -> Result<(), ArtifactError> {
        ServiceId::parse(self.service_id.as_str()).map_err(|_| ArtifactError {
            code: "SVC010".to_owned(),
            message: "service contract carries a malformed service id".to_owned(),
        })?;
        if self.name.trim().is_empty()
            || self.canonical_path.is_empty()
            || self.canonical_path.last() != Some(&self.name)
        {
            return Err(ArtifactError {
                code: "SVC011".to_owned(),
                message: "service canonical path must end with its own name".to_owned(),
            });
        }
        if self.methods.is_empty() {
            return Err(ArtifactError {
                code: "SVC012".to_owned(),
                message: "service contract must declare at least one method".to_owned(),
            });
        }
        let identity =
            ServiceIdentityKey::new(self.canonical_path.clone()).map_err(|_| ArtifactError {
                code: "SVC011".to_owned(),
                message: "service canonical path must have non-empty segments".to_owned(),
            })?;
        if ServiceId::derive(&identity) != self.service_id {
            return Err(ArtifactError {
                code: "SVC013".to_owned(),
                message: "service id does not match its canonical path".to_owned(),
            });
        }
        validate_attributes(&self.attributes, "SVC007", "service")?;

        let mut names = BTreeSet::new();
        let mut ids = BTreeSet::new();
        let mut operations = BTreeSet::new();
        for method in &self.methods {
            validate_attributes(&method.attributes, "SVC008", "service method")?;
            if method.name.trim().is_empty() {
                return Err(ArtifactError {
                    code: "SVC014".to_owned(),
                    message: "service method names must not be empty".to_owned(),
                });
            }
            if !names.insert(method.name.as_str()) {
                return Err(ArtifactError {
                    code: "SVC014".to_owned(),
                    message: "service method names must be unique".to_owned(),
                });
            }
            MethodId::parse(method.method_id.as_str()).map_err(|_| ArtifactError {
                code: "SVC015".to_owned(),
                message: "service method carries a malformed method id".to_owned(),
            })?;
            if !ids.insert(method.method_id.clone()) {
                return Err(ArtifactError {
                    code: "SVC015".to_owned(),
                    message: "service method ids must be unique".to_owned(),
                });
            }
            let expected = MethodId::derive(&MethodIdentityKey::new(
                self.service_id.clone(),
                &method.name,
            )?);
            if method.method_id != expected {
                return Err(ArtifactError {
                    code: "SVC015".to_owned(),
                    message: "service method id does not match its name and service".to_owned(),
                });
            }
            OperationId::parse(method.operation_id.as_str()).map_err(|_| ArtifactError {
                code: "SVC016".to_owned(),
                message: "service method binds a malformed operation id".to_owned(),
            })?;
            if !operations.insert(method.operation_id.clone()) {
                return Err(ArtifactError {
                    code: "SVC016".to_owned(),
                    message: "an operation cannot be bound by two methods".to_owned(),
                });
            }
            validate_explicit_reference(&method.request_type, "request")?;
            validate_explicit_reference(&method.response_type, "response")?;
        }
        Ok(())
    }
}

fn validate_attributes(
    attributes: &[ContractAttribute],
    code: &str,
    scope: &str,
) -> Result<(), ArtifactError> {
    let mut seen = BTreeSet::new();
    for attribute in attributes {
        if attribute.name.trim().is_empty() {
            return Err(ArtifactError {
                code: code.to_owned(),
                message: format!("{scope} attribute names must not be empty"),
            });
        }
        if !seen.insert((attribute.name.as_str(), attribute.syntax.as_str())) {
            return Err(ArtifactError {
                code: code.to_owned(),
                message: format!("{scope} attributes must not repeat the same name and syntax"),
            });
        }
    }
    Ok(())
}

fn validate_explicit_reference(
    canonical_type: &ResolvedCanonicalType,
    side: &str,
) -> Result<(), ArtifactError> {
    match canonical_type {
        ResolvedCanonicalType::Scalar(_) => Ok(()),
        ResolvedCanonicalType::User { path, type_id } => {
            if path.is_empty() || path.iter().any(String::is_empty) || *type_id == 0 {
                return Err(ArtifactError {
                    code: "SVC017".to_owned(),
                    message: format!(
                        "service {side} type must reference a canonical path and a durable type id"
                    ),
                });
            }
            Ok(())
        }
        ResolvedCanonicalType::Reference(inner)
        | ResolvedCanonicalType::Optional(inner)
        | ResolvedCanonicalType::List(inner) => validate_explicit_reference(inner, side),
        ResolvedCanonicalType::Generic { path, arguments } => {
            if path.is_empty() {
                return Err(ArtifactError {
                    code: "SVC017".to_owned(),
                    message: format!("service {side} generic type must be canonical"),
                });
            }
            for argument in arguments {
                if let crate::ResolvedCanonicalTypeArgument::Type(argument) = argument {
                    validate_explicit_reference(argument, side)?;
                }
            }
            Ok(())
        }
    }
}

/// Canonical fingerprint of a set of service contracts.
///
/// Services and methods are ordered by durable identity, and spans and source
/// units are excluded, so the fingerprint is stable across source units,
/// whitespace, comments and declaration order while changing on any rename,
/// retype, stream mode, operation rebinding or attribute change.
pub fn service_contract_fingerprint(services: &[ServiceContract]) -> String {
    let mut views = services.iter().collect::<Vec<_>>();
    views.sort_by(|left, right| left.service_id.cmp(&right.service_id));
    let views = views
        .into_iter()
        .map(|service| {
            let mut methods = service.methods.iter().collect::<Vec<_>>();
            methods.sort_by(|left, right| left.method_id.cmp(&right.method_id));
            CanonicalServiceContract {
                fingerprint_version: CONTRACT_FINGERPRINT_VERSION,
                service_id: &service.service_id,
                canonical_path: &service.canonical_path,
                attributes: canonical_attributes(&service.attributes),
                methods: methods
                    .into_iter()
                    .map(|method| CanonicalServiceMethod {
                        method_id: &method.method_id,
                        name: &method.name,
                        request_type: &method.request_type,
                        response_type: &method.response_type,
                        stream: method.stream,
                        operation_id: &method.operation_id,
                        attributes: canonical_attributes(&method.attributes),
                    })
                    .collect(),
            }
        })
        .collect::<Vec<_>>();
    canonical_digest(&views)
}

fn canonical_attributes(attributes: &[ContractAttribute]) -> Vec<CanonicalAttribute<'_>> {
    attributes
        .iter()
        .map(|attribute| CanonicalAttribute {
            name: &attribute.name,
            syntax: &attribute.syntax,
        })
        .collect()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CanonicalServiceContract<'a> {
    fingerprint_version: &'static str,
    service_id: &'a ServiceId,
    canonical_path: &'a [String],
    attributes: Vec<CanonicalAttribute<'a>>,
    methods: Vec<CanonicalServiceMethod<'a>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CanonicalServiceMethod<'a> {
    method_id: &'a MethodId,
    name: &'a str,
    request_type: &'a ResolvedCanonicalType,
    response_type: &'a ResolvedCanonicalType,
    stream: MethodStreamMode,
    operation_id: &'a OperationId,
    attributes: Vec<CanonicalAttribute<'a>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CanonicalAttribute<'a> {
    name: &'a str,
    syntax: &'a str,
}
