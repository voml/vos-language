use core::range::Range;

use serde::{Deserialize, Serialize};

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
    /// Fingerprint is absent until the durable identity gate is closed.
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
    /// Canonical signature key used as input to durable identity assignment.
    /// This is not itself a durable operation ID.
    pub identity_key: String,
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
        if self.envelope.schema_fingerprint.is_some()
            || self.envelope.source_units.len() != 1
            || self.envelope.source_units[0].source_unit_id != 0
        {
            return Err(ArtifactError {
                code: "ART003".to_owned(),
                message: "invalid V0 source unit or fingerprint state".to_owned(),
            });
        }
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
        if self.envelope.schema_fingerprint.is_some()
            || self.envelope.source_units.len() != 1
            || self.envelope.source_units[0].source_unit_id != 0
        {
            return Err(ArtifactError {
                code: "OPR012".to_owned(),
                message: "invalid operation projection source unit or fingerprint state".to_owned(),
            });
        }
        let source = &self.envelope.source_units[0].source;
        for operation in &self.operations {
            validate_span(&operation.span, source, "operation span")?;
            if operation.name.is_empty() {
                return Err(ArtifactError {
                    code: "OPR013".to_owned(),
                    message: "operation name cannot be empty".to_owned(),
                });
            }
            if operation.identity_key.is_empty() {
                return Err(ArtifactError {
                    code: "OPR015".to_owned(),
                    message: "operation identity key cannot be empty".to_owned(),
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
