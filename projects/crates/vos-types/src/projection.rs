use core::range::Range;

use serde::{Deserialize, Serialize};

use crate::ids::{OperationId, OperationIdentityKey, ServiceId, ServiceIdentityKey};

/// Current artifact format version for the first resolved contract envelope.
pub const CONTRACT_FORMAT_VERSION: &str = "vos-contract-v0";
/// Current VOS language contract version represented by this consumer.
pub const LANGUAGE_VERSION: &str = "vos-language-v0";
/// Canonicalization rules version for this resolved projection.
pub const CANONICALIZATION_VERSION: &str = "vos-canonical-v0";
/// Current bootstrap projection stage.
pub const PROJECTION_STAGE: &str = "syntax-projection";
/// Current operation contract projection format.
pub const OPERATION_CONTRACT_FORMAT_VERSION: &str = "vos-operation-contract-v0";
/// Operation projection stage before expression lowering and name resolution.
pub const OPERATION_PROJECTION_STAGE: &str = "operation-syntax-projection";
/// Current service contract projection format.
pub const SERVICE_CONTRACT_FORMAT_VERSION: &str = "vos-service-contract-v0";
/// Service projection stage before body lowering and name resolution.
pub const SERVICE_PROJECTION_STAGE: &str = "service-syntax-projection";

/// A source unit carried by a contract artifact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SourceUnit {
    /// Stable position within this artifact.
    pub source_unit_id: u32,
    /// Exact source text.
    pub source: String,
}

/// Version and provenance envelope for a resolved contract artifact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ContractEnvelope {
    /// Artifact processing stage.
    pub stage: String,
    /// Contract artifact format version.
    pub contract_format_version: String,
    /// VOS language contract version.
    pub language_version: String,
    /// Canonicalization rules version.
    pub canonicalization_version: String,
    /// Optional SHA-256 fingerprint of the resolved schema (64 hex chars).
    /// Projected envelopes leave this `None`; resolved contracts populate it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_fingerprint: Option<String>,
    /// Source units represented by this artifact.
    pub source_units: Vec<SourceUnit>,
}

/// A bootstrap schema projection, not a resolved semantic contract.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SchemaProjection {
    /// Artifact envelope.
    pub envelope: ContractEnvelope,
    /// Projected table and class declarations in source order.
    pub types: Vec<TypeContract>,
}

/// A resolved table or class contract.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct TypeContract {
    /// Canonical namespace-qualified type path.
    pub canonical_path: Vec<String>,
    /// Declaration kind.
    pub kind: TypeContractKind,
    /// Fields in source order. Order is not identity.
    pub fields: Vec<FieldContract>,
    /// Source span of the declaration.
    #[serde(
        serialize_with = "serialize_span",
        deserialize_with = "deserialize_span"
    )]
    pub span: Range<usize>,
}

/// Supported resolved type declaration kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TypeContractKind {
    /// Persistent object schema.
    Table,
    /// Inline value/class schema.
    Class,
}

/// A resolved field contract.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct FieldContract {
    /// Field name in canonical spelling.
    pub canonical_name: String,
    /// Canonical type syntax with names unresolved.
    pub canonical_type: CanonicalType,
    /// Semantic attributes owned by VOS.
    pub attributes: Vec<AttributeContract>,
    /// Exact default syntax when present.
    pub default_value: Option<String>,
    /// Source span of the field.
    #[serde(
        serialize_with = "serialize_span",
        deserialize_with = "deserialize_span"
    )]
    pub span: Range<usize>,
}

/// A canonical type algebra before symbol resolution.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", rename_all_fields = "camelCase")]
pub enum CanonicalType {
    /// Named builtin or user type path.
    Named(Vec<String>),
    /// Primary-key reference wrapper.
    Reference(Box<CanonicalType>),
    /// Optional wrapper.
    Optional(Box<CanonicalType>),
    /// List wrapper.
    List(Box<CanonicalType>),
    /// Generic type with canonical arguments.
    Generic {
        /// Generic name path.
        path: Vec<String>,
        /// Generic arguments in source order.
        arguments: Vec<CanonicalTypeArgument>,
    },
}

/// A canonical generic argument.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CanonicalTypeArgument {
    /// Nested type argument.
    Type(CanonicalType),
    /// Literal argument retained as canonical source text.
    Literal(String),
}

/// An attribute resolved by VOS rather than Oak.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct AttributeContract {
    /// Canonical attribute name when known.
    pub name: String,
    /// Original attribute syntax.
    pub syntax: String,
    /// Source span of the attribute.
    #[serde(
        serialize_with = "serialize_span",
        deserialize_with = "deserialize_span"
    )]
    pub span: Range<usize>,
}

/// A structured semantic diagnostic from contract resolution.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ContractDiagnostic {
    /// Stable diagnostic code within this contract version.
    pub code: String,
    /// Human-readable diagnostic message.
    pub message: String,
    /// Source span when available.
    #[serde(
        serialize_with = "serialize_optional_span",
        deserialize_with = "deserialize_optional_span"
    )]
    pub span: Option<Range<usize>>,
}

/// Operation-like declaration projected from Oak without expression lowering.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct OperationProjection {
    /// Operation declaration kind.
    pub kind: OperationKind,
    /// Operation name.
    pub name: String,
    /// Canonical signature identity key used as input to durable identity
    /// assignment. This is not itself a durable operation ID.
    pub identity: OperationIdentityKey,
    /// Durable operation identity derived from `identity`. Machine consumers
    /// (Iris, Hermes) use this field to locate an operation without re-parsing
    /// its signature string.
    pub operation_id: OperationId,
    /// Parameters projected from Oak's signature.
    pub parameters: Vec<OperationParameter>,
    /// Return type when the declaration has an arrow return type.
    pub return_type: Option<CanonicalType>,
    /// Exact signature source slice.
    pub signature: Option<SyntaxSliceContract>,
    /// Exact body source slice.
    pub body: Option<SyntaxSliceContract>,
    /// Full declaration span.
    #[serde(
        serialize_with = "serialize_span",
        deserialize_with = "deserialize_span"
    )]
    pub span: Range<usize>,
}

/// A strict, versioned operation projection artifact emitted from Oak output.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct OperationProjectionArtifact {
    /// Artifact envelope and source provenance.
    pub envelope: ContractEnvelope,
    /// Operation declarations in source order. Order is not an identity rule.
    pub operations: Vec<OperationProjection>,
}

/// Supported operation-like declaration kinds in the syntax projection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OperationKind {
    /// Query declaration.
    Query,
    /// Durable UDF declaration.
    Udf,
    /// Session-local micro declaration.
    Micro,
    /// Durable macro declaration.
    Macro,
}

/// One operation parameter projected from Oak.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct OperationParameter {
    /// Parameter name.
    pub name: String,
    /// Canonical parameter type.
    pub type_expr: CanonicalType,
    /// Full parameter span.
    #[serde(
        serialize_with = "serialize_span",
        deserialize_with = "deserialize_span"
    )]
    pub span: Range<usize>,
}

/// Exact source text and byte span retained by a syntax projection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SyntaxSliceContract {
    /// Exact source text.
    pub text: String,
    /// Source byte span.
    #[serde(
        serialize_with = "serialize_span",
        deserialize_with = "deserialize_span"
    )]
    pub span: Range<usize>,
}

impl SchemaProjection {
    /// Serializes this contract using the stable artifact field names.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Reads and validates a strict V0 schema projection artifact.
    pub fn from_json(input: &str) -> Result<Self, ArtifactError> {
        let projection: Self = serde_json::from_str(input).map_err(|error| ArtifactError {
            code: "ART001".to_owned(),
            message: error.to_string(),
        })?;
        projection.validate()?;
        Ok(projection)
    }

    fn validate(&self) -> Result<(), ArtifactError> {
        if self.envelope.stage != PROJECTION_STAGE
            || self.envelope.contract_format_version != CONTRACT_FORMAT_VERSION
            || self.envelope.language_version != LANGUAGE_VERSION
            || self.envelope.canonicalization_version != CANONICALIZATION_VERSION
        {
            return Err(ArtifactError {
                code: "ART002".to_owned(),
                message: "unsupported schema projection version or stage".to_owned(),
            });
        }
        if self.envelope.source_units.len() != 1
            || self.envelope.source_units[0].source_unit_id != 0
        {
            return Err(ArtifactError {
                code: "ART003".to_owned(),
                message: "invalid V0 source unit".to_owned(),
            });
        }
        validate_fingerprint_if_present(&self.envelope.schema_fingerprint, "ART005")?;
        let source = &self.envelope.source_units[0].source;
        for type_contract in &self.types {
            validate_span(&type_contract.span, source, "type span")?;
            for field in &type_contract.fields {
                validate_span(&field.span, source, "field span")?;
                for attribute in &field.attributes {
                    validate_span(&attribute.span, source, "attribute span")?;
                }
            }
        }
        Ok(())
    }
}

impl OperationProjectionArtifact {
    /// Serializes this operation artifact using the stable field names.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Reads and validates a strict operation projection artifact.
    pub fn from_json(input: &str) -> Result<Self, ArtifactError> {
        let artifact: Self = serde_json::from_str(input).map_err(|error| ArtifactError {
            code: "OPR010".to_owned(),
            message: error.to_string(),
        })?;
        artifact.validate()?;
        Ok(artifact)
    }

    fn validate(&self) -> Result<(), ArtifactError> {
        if self.envelope.stage != OPERATION_PROJECTION_STAGE
            || self.envelope.contract_format_version != OPERATION_CONTRACT_FORMAT_VERSION
            || self.envelope.language_version != LANGUAGE_VERSION
            || self.envelope.canonicalization_version != CANONICALIZATION_VERSION
        {
            return Err(ArtifactError {
                code: "OPR011".to_owned(),
                message: "unsupported operation projection version or stage".to_owned(),
            });
        }
        if self.envelope.source_units.len() != 1
            || self.envelope.source_units[0].source_unit_id != 0
        {
            return Err(ArtifactError {
                code: "OPR012".to_owned(),
                message: "invalid operation projection source unit".to_owned(),
            });
        }
        validate_fingerprint_if_present(&self.envelope.schema_fingerprint, "OPR018")?;
        let source = &self.envelope.source_units[0].source;
        for operation in &self.operations {
            validate_span(&operation.span, source, "operation span")?;
            if operation.name.is_empty() {
                return Err(ArtifactError {
                    code: "OPR013".to_owned(),
                    message: "operation name cannot be empty".to_owned(),
                });
            }
            if operation.identity.name.is_empty() {
                return Err(ArtifactError {
                    code: "OPR015".to_owned(),
                    message: "operation identity name cannot be empty".to_owned(),
                });
            }
            if operation.identity.name != operation.name
                || operation.identity.kind != operation.kind
            {
                return Err(ArtifactError {
                    code: "OPR015".to_owned(),
                    message: "operation identity does not match the projected declaration"
                        .to_owned(),
                });
            }
            if OperationId::parse(operation.operation_id.as_str()).is_err() {
                return Err(ArtifactError {
                    code: "OPR016".to_owned(),
                    message: "operation id is malformed".to_owned(),
                });
            }
            let expected = OperationId::derive(&operation.identity);
            if operation.operation_id != expected {
                return Err(ArtifactError {
                    code: "OPR017".to_owned(),
                    message: "operation id does not match its identity key".to_owned(),
                });
            }
            for parameter in &operation.parameters {
                validate_span(&parameter.span, source, "operation parameter span")?;
            }
            for slice in operation.signature.iter().chain(operation.body.iter()) {
                validate_span(&slice.span, source, "operation syntax slice")?;
                if source.get(slice.span) != Some(slice.text.as_str()) {
                    return Err(ArtifactError {
                        code: "OPR014".to_owned(),
                        message: "operation syntax slice does not match source".to_owned(),
                    });
                }
            }
        }
        Ok(())
    }
}

/// Service declaration projected from Oak without body lowering.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ServiceProjection {
    /// Service name.
    pub name: String,
    /// Canonical namespace-qualified path.
    pub canonical_path: Vec<String>,
    /// Opaque stable service identity derived from `canonical_path`.
    pub service_id: ServiceId,
    /// Exact body source slice.
    pub body: Option<SyntaxSliceContract>,
    /// Full declaration span.
    #[serde(
        serialize_with = "serialize_span",
        deserialize_with = "deserialize_span"
    )]
    pub span: Range<usize>,
}

/// A strict, versioned service projection artifact emitted from Oak output.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ServiceProjectionArtifact {
    /// Artifact envelope and source provenance.
    pub envelope: ContractEnvelope,
    /// Service declarations in source order. Order is not an identity rule.
    pub services: Vec<ServiceProjection>,
}

impl ServiceProjectionArtifact {
    /// Serializes this service artifact using the stable field names.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Reads and validates a strict service projection artifact.
    pub fn from_json(input: &str) -> Result<Self, ArtifactError> {
        let artifact: Self = serde_json::from_str(input).map_err(|error| ArtifactError {
            code: "SRV010".to_owned(),
            message: error.to_string(),
        })?;
        artifact.validate()?;
        Ok(artifact)
    }

    fn validate(&self) -> Result<(), ArtifactError> {
        if self.envelope.stage != SERVICE_PROJECTION_STAGE
            || self.envelope.contract_format_version != SERVICE_CONTRACT_FORMAT_VERSION
            || self.envelope.language_version != LANGUAGE_VERSION
            || self.envelope.canonicalization_version != CANONICALIZATION_VERSION
        {
            return Err(ArtifactError {
                code: "SRV011".to_owned(),
                message: "unsupported service projection version or stage".to_owned(),
            });
        }
        if self.envelope.source_units.len() != 1
            || self.envelope.source_units[0].source_unit_id != 0
        {
            return Err(ArtifactError {
                code: "SRV012".to_owned(),
                message: "invalid service projection source unit".to_owned(),
            });
        }
        validate_fingerprint_if_present(&self.envelope.schema_fingerprint, "SRV017")?;
        let source = &self.envelope.source_units[0].source;
        for service in &self.services {
            validate_span(&service.span, source, "service span")?;
            if service.name.is_empty() {
                return Err(ArtifactError {
                    code: "SRV013".to_owned(),
                    message: "service name cannot be empty".to_owned(),
                });
            }
            if ServiceId::parse(service.service_id.as_str()).is_err() {
                return Err(ArtifactError {
                    code: "SRV015".to_owned(),
                    message: "service id is malformed".to_owned(),
                });
            }
            let identity_key =
                ServiceIdentityKey::new(service.canonical_path.clone()).map_err(|_| {
                    ArtifactError {
                        code: "SRV016".to_owned(),
                        message: "service canonical path must have non-empty segments".to_owned(),
                    }
                })?;
            let expected = ServiceId::derive(&identity_key);
            if service.service_id != expected {
                return Err(ArtifactError {
                    code: "SRV016".to_owned(),
                    message: "service id does not match canonical path".to_owned(),
                });
            }
            for slice in service.body.iter() {
                validate_span(&slice.span, source, "service body syntax slice")?;
                if source.get(slice.span) != Some(slice.text.as_str()) {
                    return Err(ArtifactError {
                        code: "SRV014".to_owned(),
                        message: "service body syntax slice does not match source".to_owned(),
                    });
                }
            }
        }
        Ok(())
    }
}

/// A strict artifact decoding failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactError {
    /// Stable error code.
    pub code: String,
    /// Human-readable error message.
    pub message: String,
}

fn validate_span(span: &Range<usize>, source: &str, label: &str) -> Result<(), ArtifactError> {
    if span.start > span.end
        || span.end > source.len()
        || !source.is_char_boundary(span.start)
        || !source.is_char_boundary(span.end)
    {
        return Err(ArtifactError {
            code: "ART004".to_owned(),
            message: format!("invalid {label}"),
        });
    }
    Ok(())
}

fn validate_fingerprint_if_present(
    fingerprint: &Option<String>,
    code: &str,
) -> Result<(), ArtifactError> {
    if let Some(value) = fingerprint
        && (value.len() != 64 || !value.chars().all(|c| c.is_ascii_hexdigit()))
    {
        return Err(ArtifactError {
            code: code.to_owned(),
            message: "schema fingerprint must be 64 ascii-hex chars".to_owned(),
        });
    }
    Ok(())
}

fn deserialize_span<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Range<usize>, D::Error> {
    let span = SpanWire::deserialize(deserializer)?;
    if span.start > span.end {
        return Err(serde::de::Error::custom("span start is after span end"));
    }
    Ok((span.start..span.end).into())
}

fn deserialize_optional_span<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Range<usize>>, D::Error> {
    Option::<SpanWire>::deserialize(deserializer)?
        .map(|span| {
            if span.start > span.end {
                Err(serde::de::Error::custom("span start is after span end"))
            } else {
                Ok((span.start..span.end).into())
            }
        })
        .transpose()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SpanWire {
    start: usize,
    end: usize,
}

fn serialize_span<S: serde::Serializer>(
    span: &Range<usize>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    (span.start..span.end).serialize(serializer)
}

fn serialize_optional_span<S: serde::Serializer>(
    span: &Option<Range<usize>>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    span.as_ref()
        .map(|span| span.start..span.end)
        .serialize(serializer)
}

/// Canonical textual key of a canonical type.
///
/// The key is a deterministic, source-independent rendering used by operation
/// identity keys. It never contains whitespace, comments, spans or file
/// identity.
pub fn canonical_type_key(value: &CanonicalType) -> String {
    match value {
        CanonicalType::Named(path) => path.join("::"),
        CanonicalType::Reference(inner) => format!("&{}", canonical_type_key(inner)),
        CanonicalType::Optional(inner) => format!("{}?", canonical_type_key(inner)),
        CanonicalType::List(inner) => format!("[{}]", canonical_type_key(inner)),
        CanonicalType::Generic { path, arguments } => {
            let arguments = arguments
                .iter()
                .map(|argument| match argument {
                    CanonicalTypeArgument::Type(value) => canonical_type_key(value),
                    CanonicalTypeArgument::Literal(value) => format!("#{value}"),
                })
                .collect::<Vec<_>>()
                .join(",");
            format!("{}<{arguments}>", path.join("::"))
        }
    }
}
