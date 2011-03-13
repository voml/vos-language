use core::range::Range;

use oak_vos::{VosDeclarationKind, VosField, VosFieldAttribute, VosRoot, VosTypeArgument, VosTypeSyntax};

/// Current artifact format version for the first resolved contract envelope.
pub const CONTRACT_FORMAT_VERSION: &str = "vos-contract-v0";
/// Current VOS language contract version represented by this consumer.
pub const LANGUAGE_VERSION: &str = "vos-language-v0";
/// Canonicalization rules version for this resolved projection.
pub const CANONICALIZATION_VERSION: &str = "vos-canonical-v0";

/// A source unit carried by a contract artifact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceUnit {
    /// Stable position within this artifact.
    pub source_unit_id: u32,
    /// Exact source text.
    pub source: String,
}

/// Version and provenance envelope for a resolved contract artifact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContractEnvelope {
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

/// A resolved schema contract for the currently supported declarations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedContract {
    /// Artifact envelope.
    pub envelope: ContractEnvelope,
    /// Resolved table and class declarations in source order.
    pub types: Vec<TypeContract>,
}

/// A resolved table or class contract.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeContract {
    /// Canonical namespace-qualified type path.
    pub canonical_path: Vec<String>,
    /// Declaration kind.
    pub kind: TypeContractKind,
    /// Fields in source order. Order is not identity.
    pub fields: Vec<FieldContract>,
    /// Source span of the declaration.
    pub span: Range<usize>,
}

/// Supported resolved type declaration kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypeContractKind {
    /// Persistent object schema.
    Table,
    /// Inline value/class schema.
    Class,
}

/// A resolved field contract.
#[derive(Clone, Debug, PartialEq, Eq)]
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
    pub span: Range<usize>,
}

/// A canonical type algebra before symbol resolution.
#[derive(Clone, Debug, PartialEq, Eq)]
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CanonicalTypeArgument {
    /// Nested type argument.
    Type(CanonicalType),
    /// Literal argument retained as canonical source text.
    Literal(String),
}

/// An attribute resolved by VOS rather than Oak.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttributeContract {
    /// Canonical attribute name when known.
    pub name: String,
    /// Original attribute syntax.
    pub syntax: String,
    /// Source span of the attribute.
    pub span: Range<usize>,
}

/// A structured semantic diagnostic from contract resolution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContractDiagnostic {
    /// Stable diagnostic code within this contract version.
    pub code: String,
    /// Human-readable diagnostic message.
    pub message: String,
    /// Source span when available.
    pub span: Option<Range<usize>>,
}

/// Resolves the Oak root into the first VOS semantic contract projection.
pub fn resolve(root: &VosRoot) -> Result<ResolvedContract, Vec<ContractDiagnostic>> {
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
            _ => {}
        }
    }
    if diagnostics.is_empty() {
        Ok(ResolvedContract {
            envelope: ContractEnvelope {
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

fn resolve_field(field: &VosField) -> Result<FieldContract, ContractDiagnostic> {
    Ok(FieldContract {
        canonical_name: field.name.clone(),
        canonical_type: canonical_type(&field.type_expr),
        attributes: field.attributes.iter().map(resolve_attribute).collect(),
        default_value: field.default_value.as_ref().map(|value| value.text.clone()),
        span: field.span.clone(),
    })
}

fn resolve_attribute(attribute: &VosFieldAttribute) -> AttributeContract {
    let name = if attribute.text == "@@" {
        "primary".to_owned()
    } else if attribute.text == "@" {
        "unique".to_owned()
    } else {
        attribute.name.clone().unwrap_or_else(|| "unknown".to_owned())
    };
    AttributeContract { name, syntax: attribute.text.clone(), span: attribute.span.clone() }
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
