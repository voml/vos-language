use core::range::Range;

use oak_vos::{VosDeclarationKind, VosField, VosFieldAttribute, VosRoot, VosTypeArgument, VosTypeSyntax};
use serde::{Deserialize, Serialize};

/// Current artifact format version for the first resolved contract envelope.
pub const CONTRACT_FORMAT_VERSION: &str = "vos-contract-v0";
/// Current VOS language contract version represented by this consumer.
pub const LANGUAGE_VERSION: &str = "vos-language-v0";
/// Canonicalization rules version for this resolved projection.
pub const CANONICALIZATION_VERSION: &str = "vos-canonical-v0";
/// Current bootstrap projection stage.
pub const PROJECTION_STAGE: &str = "syntax-projection";

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
    #[serde(serialize_with = "serialize_span", deserialize_with = "deserialize_span")]
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
    #[serde(serialize_with = "serialize_span", deserialize_with = "deserialize_span")]
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
    #[serde(serialize_with = "serialize_span", deserialize_with = "deserialize_span")]
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
    #[serde(serialize_with = "serialize_optional_span", deserialize_with = "deserialize_optional_span")]
    pub span: Option<Range<usize>>,
}

/// Projects the supported Oak syntax without claiming symbol resolution.
pub fn project_schema(root: &VosRoot) -> Result<SchemaProjection, Vec<ContractDiagnostic>> {
    let mut namespace = Vec::new();
    let mut types = Vec::new();
    let mut diagnostics = Vec::new();
    for declaration in &root.declarations {
        match declaration.kind {
            VosDeclarationKind::Namespace => namespace = declaration.path.clone().unwrap_or_default(),
            VosDeclarationKind::Table | VosDeclarationKind::Class => {
                let Some(name) = declaration.name.clone() else {
                    diagnostics.push(diagnostic("VOS001", "type declaration is missing a name", Some(declaration.span.clone())));
                    continue;
                };
                let mut canonical_path = namespace.clone();
                canonical_path.push(name);
                if types.iter().any(|item: &TypeContract| item.canonical_path == canonical_path) {
                    diagnostics.push(diagnostic("VOS002", "duplicate type declaration", Some(declaration.span.clone())));
                    continue;
                }
                let mut field_names = std::collections::BTreeSet::new();
                for field in &declaration.fields {
                    if !field_names.insert(&field.name) {
                        diagnostics.push(diagnostic("VOS003", "duplicate field declaration", Some(field.name_span.clone())));
                    }
                }
                let fields = declaration.fields.iter().map(resolve_field).collect::<Result<Vec<_>, _>>();
                match fields {
                    Ok(fields) => types.push(TypeContract {
                        canonical_path,
                        kind: if declaration.kind == VosDeclarationKind::Table { TypeContractKind::Table } else { TypeContractKind::Class },
                        fields,
                        span: declaration.span.clone(),
                    }),
                    Err(error) => diagnostics.push(error),
                }
            }
            _ => diagnostics.push(diagnostic("VOS004", "declaration is not supported by this contract projection", Some(declaration.span.clone()))),
        }
    }
    if diagnostics.is_empty() {
        Ok(SchemaProjection {
            envelope: ContractEnvelope {
                stage: PROJECTION_STAGE.to_owned(),
                contract_format_version: CONTRACT_FORMAT_VERSION.to_owned(),
                language_version: LANGUAGE_VERSION.to_owned(),
                canonicalization_version: CANONICALIZATION_VERSION.to_owned(),
                schema_fingerprint: None,
                source_units: vec![SourceUnit { source_unit_id: 0, source: root.source.clone() }],
            },
            types,
        })
    } else {
        Err(diagnostics)
    }
}

impl SchemaProjection {
    /// Serializes this contract using the stable artifact field names.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Reads and validates a strict V0 schema projection artifact.
    pub fn from_json(input: &str) -> Result<Self, ArtifactError> {
        let projection: Self = serde_json::from_str(input).map_err(|error| ArtifactError { code: "ART001".to_owned(), message: error.to_string() })?;
        projection.validate()?;
        Ok(projection)
    }

    fn validate(&self) -> Result<(), ArtifactError> {
        if self.envelope.stage != PROJECTION_STAGE || self.envelope.contract_format_version != CONTRACT_FORMAT_VERSION || self.envelope.language_version != LANGUAGE_VERSION || self.envelope.canonicalization_version != CANONICALIZATION_VERSION {
            return Err(ArtifactError { code: "ART002".to_owned(), message: "unsupported schema projection version or stage".to_owned() });
        }
        if self.envelope.schema_fingerprint.is_some() || self.envelope.source_units.len() != 1 || self.envelope.source_units[0].source_unit_id != 0 {
            return Err(ArtifactError { code: "ART003".to_owned(), message: "invalid V0 source unit or fingerprint state".to_owned() });
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

/// A strict artifact decoding failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactError {
    /// Stable error code.
    pub code: String,
    /// Human-readable error message.
    pub message: String,
}

fn validate_span(span: &Range<usize>, source: &str, label: &str) -> Result<(), ArtifactError> {
    if span.start > span.end || span.end > source.len() || !source.is_char_boundary(span.start) || !source.is_char_boundary(span.end) {
        return Err(ArtifactError { code: "ART004".to_owned(), message: format!("invalid {label}") });
    }
    Ok(())
}

fn deserialize_span<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Range<usize>, D::Error> {
    let span = SpanWire::deserialize(deserializer)?;
    if span.start > span.end {
        return Err(serde::de::Error::custom("span start is after span end"));
    }
    Ok((span.start..span.end).into())
}

fn deserialize_optional_span<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Option<Range<usize>>, D::Error> {
    Option::<SpanWire>::deserialize(deserializer)?.map(|span| {
        if span.start > span.end {
            Err(serde::de::Error::custom("span start is after span end"))
        } else {
            Ok((span.start..span.end).into())
        }
    }).transpose()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SpanWire {
    start: usize,
    end: usize,
}

fn resolve_field(field: &VosField) -> Result<FieldContract, ContractDiagnostic> {
    Ok(FieldContract {
        canonical_name: field.name.clone(),
        canonical_type: canonical_type(&field.type_expr),
        attributes: field.attributes.iter().map(resolve_attribute).collect::<Result<Vec<_>, _>>()?,
        default_value: field.default_value.as_ref().map(|value| value.text.clone()),
        span: field.span.clone(),
    })
}

fn resolve_attribute(attribute: &VosFieldAttribute) -> Result<AttributeContract, ContractDiagnostic> {
    let name = if attribute.text == "@@" {
        "primary".to_owned()
    } else if attribute.text == "@" {
        "unique".to_owned()
    } else {
        let Some(name) = &attribute.name else {
            return Err(diagnostic("VOS005", "attribute has no structured name", Some(attribute.span.clone())));
        };
        if attribute.text != format!("[{name}]") {
            return Err(diagnostic("VOS005", "attribute group or arguments require structured Oak attribute output", Some(attribute.span.clone())));
        }
        name.clone()
    };
    Ok(AttributeContract { name, syntax: attribute.text.clone(), span: attribute.span.clone() })
}

fn serialize_span<S: serde::Serializer>(span: &Range<usize>, serializer: S) -> Result<S::Ok, S::Error> {
    (span.start..span.end).serialize(serializer)
}

fn serialize_optional_span<S: serde::Serializer>(span: &Option<Range<usize>>, serializer: S) -> Result<S::Ok, S::Error> {
    span.as_ref().map(|span| span.start..span.end).serialize(serializer)
}

fn canonical_type(value: &VosTypeSyntax) -> CanonicalType {
    match value {
        VosTypeSyntax::Named { path, .. } => CanonicalType::Named(path.clone()),
        VosTypeSyntax::Reference { target, .. } => CanonicalType::Reference(Box::new(canonical_type(target))),
        VosTypeSyntax::Optional { inner, .. } => CanonicalType::Optional(Box::new(canonical_type(inner))),
        VosTypeSyntax::List { element, .. } => CanonicalType::List(Box::new(canonical_type(element))),
        VosTypeSyntax::Generic { path, arguments, .. } => CanonicalType::Generic {
            path: path.clone(),
            arguments: arguments
                .iter()
                .map(|argument| match argument {
                    VosTypeArgument::Type(value) => CanonicalTypeArgument::Type(canonical_type(value)),
                    VosTypeArgument::Literal(value) => CanonicalTypeArgument::Literal(value.text.clone()),
                })
                .collect(),
        },
    }
}

fn diagnostic(code: &str, message: &str, span: Option<Range<usize>>) -> ContractDiagnostic {
    ContractDiagnostic { code: code.to_owned(), message: message.to_owned(), span }
}
