use core::range::Range;
use oak_vos::{
    VosDeclarationKind, VosField, VosFieldAttribute, VosRoot, VosTypeArgument, VosTypeSyntax,
};
use vos_types::*;

/// Projects Oak operation declarations without parsing their bodies.
pub fn project_operations(
    root: &VosRoot,
) -> Result<Vec<OperationProjection>, Vec<ContractDiagnostic>> {
    let mut operations = Vec::new();
    let mut diagnostics = Vec::new();
    for declaration in &root.declarations {
        let kind = match declaration.kind {
            VosDeclarationKind::Query => OperationKind::Query,
            VosDeclarationKind::Udf => OperationKind::Udf,
            VosDeclarationKind::Micro => OperationKind::Micro,
            VosDeclarationKind::Macro => OperationKind::Macro,
            _ => continue,
        };
        let Some(name) = declaration.name.clone() else {
            diagnostics.push(diagnostic(
                "OPR001",
                "operation declaration is missing a name",
                Some(declaration.span),
            ));
            continue;
        };
        let parameters = declaration
            .parameters
            .iter()
            .map(|parameter| OperationParameter {
                name: parameter.name.clone(),
                type_expr: canonical_type(&parameter.type_expr),
                span: parameter.span,
            })
            .collect::<Vec<_>>();
        let identity = match operation_identity(
            kind,
            &name,
            &parameters,
            declaration.return_type_expr.as_ref(),
        ) {
            Ok(identity) => identity,
            Err(error) => {
                diagnostics.push(diagnostic(
                    error.code.as_str(),
                    &error.message,
                    Some(declaration.span),
                ));
                continue;
            }
        };
        operations.push(OperationProjection {
            kind,
            name,
            operation_id: OperationId::derive(&identity),
            identity,
            parameters,
            return_type: declaration.return_type_expr.as_ref().map(canonical_type),
            signature: declaration.signature.as_ref().map(slice_contract),
            body: declaration.body.as_ref().map(slice_contract),
            span: declaration.span,
        });
    }
    if diagnostics.is_empty() {
        Ok(operations)
    } else {
        Err(diagnostics)
    }
}

fn operation_identity(
    kind: OperationKind,
    name: &str,
    parameters: &[OperationParameter],
    return_type: Option<&VosTypeSyntax>,
) -> Result<OperationIdentityKey, vos_types::ArtifactError> {
    let parameters = parameters
        .iter()
        .map(|parameter| {
            OperationParameterKey::new(&parameter.name, &canonical_type_key(&parameter.type_expr))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let result = return_type
        .map(|value| canonical_type_key(&canonical_type(value)))
        .unwrap_or_else(|| "unit".to_owned());
    OperationIdentityKey::new(kind, name, parameters, &result)
}

/// Projects Oak operation declarations into a strict versioned artifact.
pub fn project_operations_artifact(
    root: &VosRoot,
) -> Result<OperationProjectionArtifact, Vec<ContractDiagnostic>> {
    Ok(OperationProjectionArtifact {
        envelope: ContractEnvelope {
            stage: OPERATION_PROJECTION_STAGE.to_owned(),
            contract_format_version: OPERATION_CONTRACT_FORMAT_VERSION.to_owned(),
            language_version: LANGUAGE_VERSION.to_owned(),
            canonicalization_version: CANONICALIZATION_VERSION.to_owned(),
            schema_fingerprint: None,
            source_units: vec![SourceUnit {
                source_unit_id: 0,
                source: root.source.clone(),
            }],
        },
        operations: project_operations(root)?,
    })
}

/// Projects Oak service declarations without lowering their bodies.
pub fn project_services(root: &VosRoot) -> Result<Vec<ServiceProjection>, Vec<ContractDiagnostic>> {
    let mut namespace = Vec::new();
    let mut services = Vec::new();
    let mut diagnostics = Vec::new();
    for declaration in &root.declarations {
        match declaration.kind {
            VosDeclarationKind::Namespace => {
                namespace = declaration.path.clone().unwrap_or_default()
            }
            VosDeclarationKind::Service => {
                let Some(name) = declaration.name.clone() else {
                    diagnostics.push(diagnostic(
                        "SRV001",
                        "service declaration is missing a name",
                        Some(declaration.span),
                    ));
                    continue;
                };
                let mut canonical_path = namespace.clone();
                canonical_path.push(name.clone());
                if services
                    .iter()
                    .any(|item: &ServiceProjection| item.canonical_path == canonical_path)
                {
                    diagnostics.push(diagnostic(
                        "SRV002",
                        "duplicate service declaration",
                        Some(declaration.span),
                    ));
                    continue;
                }
                let identity = match ServiceIdentityKey::new(canonical_path.clone()) {
                    Ok(identity) => identity,
                    Err(error) => {
                        diagnostics.push(diagnostic(
                            error.code.as_str(),
                            &error.message,
                            Some(declaration.span),
                        ));
                        continue;
                    }
                };
                services.push(ServiceProjection {
                    name,
                    canonical_path,
                    service_id: ServiceId::derive(&identity),
                    body: declaration.body.as_ref().map(slice_contract),
                    span: declaration.span,
                });
            }
            _ => continue,
        }
    }
    if diagnostics.is_empty() {
        Ok(services)
    } else {
        Err(diagnostics)
    }
}

/// Projects Oak service declarations into a strict versioned artifact.
pub fn project_services_artifact(
    root: &VosRoot,
) -> Result<ServiceProjectionArtifact, Vec<ContractDiagnostic>> {
    Ok(ServiceProjectionArtifact {
        envelope: ContractEnvelope {
            stage: SERVICE_PROJECTION_STAGE.to_owned(),
            contract_format_version: SERVICE_CONTRACT_FORMAT_VERSION.to_owned(),
            language_version: LANGUAGE_VERSION.to_owned(),
            canonicalization_version: CANONICALIZATION_VERSION.to_owned(),
            schema_fingerprint: None,
            source_units: vec![SourceUnit {
                source_unit_id: 0,
                source: root.source.clone(),
            }],
        },
        services: project_services(root)?,
    })
}

fn slice_contract(slice: &oak_vos::VosSyntaxSlice) -> SyntaxSliceContract {
    SyntaxSliceContract {
        text: slice.text.clone(),
        span: slice.span,
    }
}

/// Projects the supported Oak syntax without claiming symbol resolution.
pub fn project_schema(root: &VosRoot) -> Result<SchemaProjection, Vec<ContractDiagnostic>> {
    let mut namespace = Vec::new();
    let mut types = Vec::new();
    let mut diagnostics = Vec::new();
    for declaration in &root.declarations {
        match declaration.kind {
            VosDeclarationKind::Namespace => {
                namespace = declaration.path.clone().unwrap_or_default()
            }
            VosDeclarationKind::Table | VosDeclarationKind::Class => {
                let Some(name) = declaration.name.clone() else {
                    diagnostics.push(diagnostic(
                        "VOS001",
                        "type declaration is missing a name",
                        Some(declaration.span),
                    ));
                    continue;
                };
                let mut canonical_path = namespace.clone();
                canonical_path.push(name);
                if types
                    .iter()
                    .any(|item: &TypeContract| item.canonical_path == canonical_path)
                {
                    diagnostics.push(diagnostic(
                        "VOS002",
                        "duplicate type declaration",
                        Some(declaration.span),
                    ));
                    continue;
                }
                let mut field_names = std::collections::BTreeSet::new();
                for field in &declaration.fields {
                    if !field_names.insert(&field.name) {
                        diagnostics.push(diagnostic(
                            "VOS003",
                            "duplicate field declaration",
                            Some(field.name_span),
                        ));
                    }
                }
                let fields = declaration
                    .fields
                    .iter()
                    .map(resolve_field)
                    .collect::<Result<Vec<_>, _>>();
                match fields {
                    Ok(fields) => types.push(TypeContract {
                        canonical_path,
                        kind: if declaration.kind == VosDeclarationKind::Table {
                            TypeContractKind::Table
                        } else {
                            TypeContractKind::Class
                        },
                        fields,
                        span: declaration.span,
                    }),
                    Err(error) => diagnostics.push(error),
                }
            }
            // Service and operation-like declarations have their own projection
            // functions and are intentionally skipped here.
            VosDeclarationKind::Service
            | VosDeclarationKind::Query
            | VosDeclarationKind::Udf
            | VosDeclarationKind::Micro
            | VosDeclarationKind::Macro => continue,
            _ => diagnostics.push(diagnostic(
                "VOS004",
                "declaration is not supported by this contract projection",
                Some(declaration.span),
            )),
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
                source_units: vec![SourceUnit {
                    source_unit_id: 0,
                    source: root.source.clone(),
                }],
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
        attributes: field
            .attributes
            .iter()
            .map(resolve_attribute)
            .collect::<Result<Vec<_>, _>>()?,
        default_value: field.default_value.as_ref().map(|value| value.text.clone()),
        span: field.span,
    })
}

fn resolve_attribute(
    attribute: &VosFieldAttribute,
) -> Result<AttributeContract, ContractDiagnostic> {
    let name = if attribute.text == "@@" {
        "primary".to_owned()
    } else if attribute.text == "@" {
        "unique".to_owned()
    } else {
        let Some(name) = &attribute.name else {
            return Err(diagnostic(
                "VOS005",
                "attribute has no structured name",
                Some(attribute.span),
            ));
        };
        if attribute.text != format!("[{name}]") {
            return Err(diagnostic(
                "VOS005",
                "attribute group or arguments require structured Oak attribute output",
                Some(attribute.span),
            ));
        }
        name.clone()
    };
    Ok(AttributeContract {
        name,
        syntax: attribute.text.clone(),
        span: attribute.span,
    })
}

fn canonical_type(value: &VosTypeSyntax) -> CanonicalType {
    match value {
        VosTypeSyntax::Named { path, .. } => CanonicalType::Named(path.clone()),
        VosTypeSyntax::Reference { target, .. } => {
            CanonicalType::Reference(Box::new(canonical_type(target)))
        }
        VosTypeSyntax::Optional { inner, .. } => {
            CanonicalType::Optional(Box::new(canonical_type(inner)))
        }
        VosTypeSyntax::List { element, .. } => {
            CanonicalType::List(Box::new(canonical_type(element)))
        }
        VosTypeSyntax::Generic {
            path, arguments, ..
        } => CanonicalType::Generic {
            path: path.clone(),
            arguments: arguments
                .iter()
                .map(|argument| match argument {
                    VosTypeArgument::Type(value) => {
                        CanonicalTypeArgument::Type(canonical_type(value))
                    }
                    VosTypeArgument::Literal(value) => {
                        CanonicalTypeArgument::Literal(value.text.clone())
                    }
                })
                .collect(),
        },
    }
}

fn diagnostic(code: &str, message: &str, span: Option<Range<usize>>) -> ContractDiagnostic {
    ContractDiagnostic {
        code: code.to_owned(),
        message: message.to_owned(),
        span,
    }
}
