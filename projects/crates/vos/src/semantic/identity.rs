use std::collections::{BTreeMap, BTreeSet};

use core::range::Range;

use vos_types::*;

/// Binds a reviewable identity manifest to a schema projection.
pub fn bind_identity(
    projection: &SchemaProjection,
    manifest: &IdentityManifest,
) -> Result<IdentityBoundProjection, Vec<ContractDiagnostic>> {
    let mut diagnostics = Vec::new();
    if manifest.format_version != IDENTITY_MANIFEST_VERSION {
        diagnostics.push(diagnostic("ID001", "unsupported identity manifest version"));
    }

    let mut type_index = BTreeMap::new();
    let mut type_ids = BTreeSet::new();
    let mut field_ids = BTreeSet::new();
    for identity in &manifest.types {
        if identity.type_id == 0 || !type_ids.insert(identity.type_id) {
            diagnostics.push(diagnostic("ID002", "type ID must be non-zero and unique"));
        }
        if identity.canonical_path.is_empty()
            || identity.canonical_path.iter().any(String::is_empty)
            || type_index
                .insert(identity.canonical_path.clone(), identity)
                .is_some()
        {
            diagnostics.push(diagnostic(
                "ID003",
                "empty or duplicate type path in identity manifest",
            ));
        }
        validate_field_manifest(identity, &mut diagnostics);
        for field in &identity.fields {
            if !field_ids.insert(field.field_id) {
                diagnostics.push(diagnostic(
                    "ID009",
                    "field ID must be unique throughout the manifest",
                ));
            }
        }
    }

    let projection_paths = projection
        .types
        .iter()
        .map(|item| item.canonical_path.clone())
        .collect::<BTreeSet<_>>();
    let manifest_paths = type_index.keys().cloned().collect::<BTreeSet<_>>();
    for missing in projection_paths.difference(&manifest_paths) {
        diagnostics.push(diagnostic(
            "ID004",
            &format!("missing identity for type {}", missing.join("::")),
        ));
    }
    for unknown in manifest_paths.difference(&projection_paths) {
        diagnostics.push(diagnostic(
            "ID005",
            &format!("identity refers to unknown type {}", unknown.join("::")),
        ));
    }

    let mut bound_types = Vec::new();
    if diagnostics.is_empty() {
        for projected in &projection.types {
            let identity = type_index
                .get(&projected.canonical_path)
                .expect("validated type path");
            let fields = identity
                .fields
                .iter()
                .map(|field| (field.canonical_name.clone(), field))
                .collect::<BTreeMap<_, _>>();
            let projected_names = projected
                .fields
                .iter()
                .map(|field| field.canonical_name.clone())
                .collect::<BTreeSet<_>>();
            let identity_names = fields.keys().cloned().collect::<BTreeSet<_>>();
            for missing in projected_names.difference(&identity_names) {
                diagnostics.push(diagnostic(
                    "ID006",
                    &format!("missing identity for field {}", missing),
                ));
            }
            for unknown in identity_names.difference(&projected_names) {
                diagnostics.push(diagnostic(
                    "ID007",
                    &format!("identity refers to unknown field {}", unknown),
                ));
            }
            if identity.kind != projected.kind {
                diagnostics.push(diagnostic("ID008", "identity kind mismatch"));
            }
            if diagnostics.is_empty() {
                bound_types.push(BoundTypeContract {
                    type_id: identity.type_id,
                    canonical_path: projected.canonical_path.clone(),
                    kind: projected.kind,
                    fields: projected
                        .fields
                        .iter()
                        .map(|field| {
                            let identity = fields
                                .get(&field.canonical_name)
                                .expect("validated field name");
                            BoundFieldContract {
                                field_id: identity.field_id,
                                virtual_field_index: identity.virtual_field_index,
                                canonical_name: field.canonical_name.clone(),
                                canonical_type: field.canonical_type.clone(),
                                attributes: field.attributes.clone(),
                                default_value: field.default_value.clone(),
                            }
                        })
                        .collect(),
                });
            }
        }
    }
    if diagnostics.is_empty() {
        Ok(IdentityBoundProjection {
            manifest_version: manifest.format_version.clone(),
            types: bound_types,
        })
    } else {
        Err(diagnostics)
    }
}

/// Compares two explicitly bound snapshots without assigning or reusing IDs.
pub fn compare_identity(
    previous: &IdentityBoundProjection,
    current: &IdentityBoundProjection,
) -> Result<IdentityEvolution, Vec<ContractDiagnostic>> {
    let mut diagnostics = Vec::new();
    let previous_types = previous
        .types
        .iter()
        .map(|item| (item.type_id, item))
        .collect::<BTreeMap<_, _>>();
    let current_types = current
        .types
        .iter()
        .map(|item| (item.type_id, item))
        .collect::<BTreeMap<_, _>>();

    for previous_type in &previous.types {
        if let Some(current_type) = current
            .types
            .iter()
            .find(|item| item.canonical_path == previous_type.canonical_path)
            && current_type.type_id != previous_type.type_id
        {
            diagnostics.push(diagnostic("ID012", "type path changed durable ID"));
        }
        for previous_field in &previous_type.fields {
            if let Some(current_type) = current
                .types
                .iter()
                .find(|item| item.type_id == previous_type.type_id)
            {
                if let Some(current_field) = current_type
                    .fields
                    .iter()
                    .find(|item| item.field_id == previous_field.field_id)
                    && current_field.virtual_field_index != previous_field.virtual_field_index
                {
                    diagnostics.push(diagnostic(
                        "ID027",
                        "assigned virtual field index cannot change",
                    ));
                }
                if let Some(current_field) = current_type
                    .fields
                    .iter()
                    .find(|item| item.canonical_name == previous_field.canonical_name)
                    && current_field.field_id != previous_field.field_id
                {
                    diagnostics.push(diagnostic("ID013", "field name changed durable ID"));
                }
            }
        }
    }
    for current_type in &current.types {
        for current_field in &current_type.fields {
            if let Some(previous_type) = previous.types.iter().find(|item| {
                item.fields
                    .iter()
                    .any(|field| field.field_id == current_field.field_id)
            }) && previous_type.type_id != current_type.type_id
            {
                diagnostics.push(diagnostic("ID014", "field identity moved between types"));
            }
        }
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let mut changes = Vec::new();
    let type_ids = previous_types
        .keys()
        .chain(current_types.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    for type_id in type_ids {
        match (previous_types.get(&type_id), current_types.get(&type_id)) {
            (None, Some(current_type)) => changes.push(IdentityChange::TypeAdded {
                type_id,
                canonical_path: current_type.canonical_path.clone(),
            }),
            (Some(previous_type), None) => {
                changes.push(IdentityChange::TypeRemoved {
                    type_id,
                    canonical_path: previous_type.canonical_path.clone(),
                });
                for field in &previous_type.fields {
                    changes.push(IdentityChange::FieldRemoved {
                        type_id,
                        field_id: field.field_id,
                        canonical_name: field.canonical_name.clone(),
                        virtual_field_index: field.virtual_field_index,
                    });
                }
            }
            (Some(previous_type), Some(current_type)) => {
                if previous_type.canonical_path != current_type.canonical_path {
                    changes.push(IdentityChange::TypeRenamed {
                        type_id,
                        from: previous_type.canonical_path.clone(),
                        to: current_type.canonical_path.clone(),
                    });
                }
                if previous_type.kind != current_type.kind {
                    changes.push(IdentityChange::TypeKindChanged {
                        type_id,
                        from: previous_type.kind,
                        to: current_type.kind,
                    });
                }
                compare_fields(
                    type_id,
                    &previous_type.fields,
                    &current_type.fields,
                    &mut changes,
                );
            }
            (None, None) => unreachable!(),
        }
    }
    Ok(IdentityEvolution {
        from_manifest_version: previous.manifest_version.clone(),
        to_manifest_version: current.manifest_version.clone(),
        from_fingerprint: schema_fingerprint(previous),
        to_fingerprint: schema_fingerprint(current),
        changes,
    })
}

/// Resolves an identity-bound projection into the strict resolved contract.
pub fn resolve_contract_from_bound(
    bound: &IdentityBoundProjection,
) -> Result<ResolvedContract, Vec<ContractDiagnostic>> {
    resolve_contract_from_units(&[bound])
}

/// Resolves a deterministic collection of already-bound source units into one
/// strict resolved contract.
pub fn resolve_contract_from_units(
    units: &[&IdentityBoundProjection],
) -> Result<ResolvedContract, Vec<ContractDiagnostic>> {
    let mut diagnostics = Vec::new();
    let manifest_version = units
        .first()
        .map(|unit| unit.manifest_version.clone())
        .unwrap_or_else(|| IDENTITY_MANIFEST_VERSION.to_owned());
    let mut types = Vec::new();
    let mut type_paths = BTreeMap::new();
    let mut seen_type_ids = BTreeMap::new();
    let mut field_ids = BTreeSet::new();
    for unit in units {
        if unit.manifest_version != manifest_version {
            diagnostics.push(diagnostic(
                "RES003",
                "identity manifest version differs across source units",
            ));
        }
        for item in &unit.types {
            if type_paths
                .insert(item.canonical_path.clone(), item)
                .is_some()
            {
                diagnostics.push(diagnostic(
                    "RES004",
                    "duplicate type path across source units",
                ));
            }
            if item.type_id == 0 {
                diagnostics.push(diagnostic("RES005", "zero type ID in source unit"));
            } else if seen_type_ids
                .insert(item.type_id, item.canonical_path.clone())
                .is_some()
            {
                diagnostics.push(diagnostic(
                    "RES007",
                    "duplicate type ID across source units",
                ));
            }
            for field in &item.fields {
                if !field_ids.insert(field.field_id) {
                    diagnostics.push(diagnostic(
                        "RES006",
                        "duplicate field ID across source units",
                    ));
                }
            }
            types.push(item);
        }
    }
    let type_ids = type_paths
        .iter()
        .map(|(path, item)| (path.clone(), item.type_id))
        .collect::<BTreeMap<_, _>>();
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let types = types
        .into_iter()
        .map(|item| {
            let fields = item
                .fields
                .iter()
                .filter_map(|field| {
                    // Bound field contracts carry no source span, so a schema
                    // reference diagnostic stays location-free.
                    let canonical_type = resolve_type(
                        &field.canonical_type,
                        &item.canonical_path[..item.canonical_path.len().saturating_sub(1)],
                        &type_ids,
                        None,
                        &mut diagnostics,
                    )?;
                    Some(ResolvedFieldContract {
                        field_id: field.field_id,
                        virtual_field_index: field.virtual_field_index,
                        canonical_name: field.canonical_name.clone(),
                        canonical_type,
                        attributes: field.attributes.clone(),
                        default_value: field.default_value.clone(),
                    })
                })
                .collect();
            ResolvedTypeContract {
                type_id: item.type_id,
                canonical_path: item.canonical_path.clone(),
                kind: item.kind,
                fields,
            }
        })
        .collect();
    if diagnostics.is_empty() {
        Ok(resolved_contract(manifest_version, types))
    } else {
        Err(diagnostics)
    }
}

/// Builds the strict resolved contract from one bound projection.
pub fn resolve_contract(
    projection: &SchemaProjection,
    manifest: &IdentityManifest,
) -> Result<ResolvedContract, Vec<ContractDiagnostic>> {
    let bound = bind_identity(projection, manifest)?;
    resolve_contract_from_bound(&bound)
}

/// Wraps already-resolved types into the versioned contract envelope.
fn resolved_contract(
    identity_manifest_version: String,
    types: Vec<ResolvedTypeContract>,
) -> ResolvedContract {
    let schema_fingerprint = resolved_schema_fingerprint(&types);
    ResolvedContract {
        format_version: RESOLVED_CONTRACT_FORMAT_VERSION.to_owned(),
        identity_manifest_version,
        schema_fingerprint,
        types,
    }
}

pub(crate) fn resolve_type(
    ty: &CanonicalType,
    namespace: &[String],
    type_ids: &BTreeMap<Vec<String>, u64>,
    span: Option<Range<usize>>,
    diagnostics: &mut Vec<ContractDiagnostic>,
) -> Option<ResolvedCanonicalType> {
    match ty {
        CanonicalType::Named(path) => {
            let mut candidates = Vec::new();
            if let Some(type_id) = type_ids.get(path) {
                candidates.push((path.clone(), *type_id));
            }
            if !path.is_empty() {
                let mut relative = namespace.to_vec();
                relative.extend(path.iter().cloned());
                if let Some(type_id) = type_ids.get(&relative)
                    && !candidates
                        .iter()
                        .any(|(_, candidate_id)| candidate_id == type_id)
                {
                    candidates.push((relative, *type_id));
                }
            }
            if candidates.len() > 1 {
                diagnostics.push(diagnostic_with_span(
                    "RES002",
                    &format!("ambiguous type {}", path.join("::")),
                    span,
                ));
                return None;
            }
            if let Some((canonical_path, type_id)) = candidates.first() {
                Some(ResolvedCanonicalType::User {
                    path: canonical_path.clone(),
                    type_id: *type_id,
                })
            } else if path.len() == 1 && path[0] == "file" {
                Some(ResolvedCanonicalType::File)
            } else if path.len() == 1
                && let Some(scalar) = CanonicalScalar::from_source_name(path[0].as_str())
            {
                Some(ResolvedCanonicalType::Scalar(scalar))
            } else {
                diagnostics.push(diagnostic_with_span(
                    "RES001",
                    &format!("unknown type {}", path.join("::")),
                    span,
                ));
                None
            }
        }
        CanonicalType::Reference(inner) => Some(ResolvedCanonicalType::Reference(Box::new(
            resolve_type(inner, namespace, type_ids, span, diagnostics)?,
        ))),
        CanonicalType::Optional(inner) => Some(ResolvedCanonicalType::Optional(Box::new(
            resolve_type(inner, namespace, type_ids, span, diagnostics)?,
        ))),
        CanonicalType::List(inner) => Some(ResolvedCanonicalType::List(Box::new(resolve_type(
            inner,
            namespace,
            type_ids,
            span,
            diagnostics,
        )?))),
        CanonicalType::Generic { path, arguments } => {
            if path == &["DateTime".to_owned()]
                && arguments.len() == 1
                && matches!(&arguments[0], CanonicalTypeArgument::Type(CanonicalType::Named(inner)) if inner == &["UTC".to_owned()])
            {
                Some(ResolvedCanonicalType::Scalar(CanonicalScalar::DateTime))
            } else if path == &["bytes".to_owned()] {
                diagnostics.push(diagnostic_with_span(
                    "RES017",
                    "bytes is a scalar and does not take type arguments",
                    span,
                ));
                None
            } else if path == &["file".to_owned()] {
                diagnostics.push(diagnostic_with_span(
                    "RES018",
                    "file is a logical type and does not take type arguments",
                    span,
                ));
                None
            } else {
                Some(ResolvedCanonicalType::Generic {
                    path: path.clone(),
                    arguments: arguments
                        .iter()
                        .map(|argument| match argument {
                            CanonicalTypeArgument::Type(ty) => {
                                resolve_type(ty, namespace, type_ids, span, diagnostics)
                                    .map(ResolvedCanonicalTypeArgument::Type)
                            }
                            CanonicalTypeArgument::Literal(value) => {
                                Some(ResolvedCanonicalTypeArgument::Literal(value.clone()))
                            }
                        })
                        .collect::<Option<Vec<_>>>()?,
                })
            }
        }
    }
}

/// Applies an explicitly bound snapshot to durable identity history.
pub fn evolve_identity(
    previous: &IdentityHistory,
    current: IdentityBoundProjection,
) -> Result<IdentityHistory, Vec<ContractDiagnostic>> {
    let mut diagnostics = Vec::new();
    if previous.manifest_version != current.manifest_version {
        diagnostics.push(diagnostic("ID017", "identity manifest version changed"));
    }
    let evolution = match compare_identity(&previous.snapshot, &current) {
        Ok(evolution) => evolution,
        Err(mut errors) => {
            diagnostics.append(&mut errors);
            return Err(diagnostics);
        }
    };
    let retired_type_ids = previous
        .retired_types
        .iter()
        .map(|item| item.type_id)
        .collect::<BTreeSet<_>>();
    let retired_field_ids = previous
        .retired_fields
        .iter()
        .map(|item| item.field_id)
        .collect::<BTreeSet<_>>();
    if current
        .types
        .iter()
        .any(|item| retired_type_ids.contains(&item.type_id))
    {
        diagnostics.push(diagnostic("ID015", "retired type ID cannot be reused"));
    }
    if current
        .types
        .iter()
        .flat_map(|item| item.fields.iter())
        .any(|item| retired_field_ids.contains(&item.field_id))
    {
        diagnostics.push(diagnostic("ID016", "retired field ID cannot be reused"));
    }
    let retired_slots = previous
        .retired_fields
        .iter()
        .filter_map(|item| item.virtual_field_index.map(|slot| (item.type_id, slot)))
        .collect::<BTreeSet<_>>();
    if current
        .types
        .iter()
        .flat_map(|item| {
            item.fields
                .iter()
                .filter_map(move |field| field.virtual_field_index.map(|slot| (item.type_id, slot)))
        })
        .any(|slot| retired_slots.contains(&slot))
    {
        diagnostics.push(diagnostic(
            "ID026",
            "retired virtual field slot cannot be reused",
        ));
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let changed = evolution.from_fingerprint != evolution.to_fingerprint;
    let revision = if changed {
        previous
            .revision
            .checked_add(1)
            .ok_or_else(|| vec![diagnostic("ID018", "identity revision exhausted")])?
    } else {
        previous.revision
    };
    let layout_changed = evolution.changes.iter().any(|change| {
        matches!(
            change,
            IdentityChange::TypeAdded { .. }
                | IdentityChange::TypeRemoved { .. }
                | IdentityChange::FieldAdded { .. }
                | IdentityChange::FieldRemoved { .. }
                | IdentityChange::FieldReordered { .. }
        )
    });
    let layout_epoch = if layout_changed {
        previous
            .layout_epoch
            .checked_add(1)
            .ok_or_else(|| vec![diagnostic("ID019", "identity layout epoch exhausted")])?
    } else {
        previous.layout_epoch
    };
    let mut retired_types = previous.retired_types.clone();
    let mut retired_fields = previous.retired_fields.clone();
    for change in &evolution.changes {
        match change {
            IdentityChange::TypeRemoved {
                type_id,
                canonical_path,
            } => retired_types.push(RetiredTypeIdentity {
                type_id: *type_id,
                canonical_path: canonical_path.clone(),
                retired_at_revision: revision,
            }),
            IdentityChange::FieldRemoved {
                type_id,
                field_id,
                canonical_name,
                virtual_field_index,
            } => retired_fields.push(RetiredFieldIdentity {
                field_id: *field_id,
                type_id: *type_id,
                canonical_name: canonical_name.clone(),
                virtual_field_index: *virtual_field_index,
                retired_at_revision: revision,
            }),
            _ => {}
        }
    }
    retired_types.sort_by_key(|item| item.type_id);
    retired_fields.sort_by_key(|item| item.field_id);
    Ok(IdentityHistory {
        format_version: IDENTITY_HISTORY_FORMAT_VERSION.to_owned(),
        manifest_version: current.manifest_version.clone(),
        revision,
        layout_epoch,
        snapshot: current,
        retired_types,
        retired_fields,
    })
}

fn compare_fields(
    type_id: u64,
    previous: &[BoundFieldContract],
    current: &[BoundFieldContract],
    changes: &mut Vec<IdentityChange>,
) {
    let previous_fields = previous
        .iter()
        .map(|item| (item.field_id, item))
        .collect::<BTreeMap<_, _>>();
    let current_fields = current
        .iter()
        .map(|item| (item.field_id, item))
        .collect::<BTreeMap<_, _>>();
    let field_ids = previous_fields
        .keys()
        .chain(current_fields.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    for field_id in field_ids {
        match (
            previous_fields.get(&field_id),
            current_fields.get(&field_id),
        ) {
            (None, Some(field)) => changes.push(IdentityChange::FieldAdded {
                type_id,
                field_id,
                canonical_name: field.canonical_name.clone(),
                virtual_field_index: field.virtual_field_index,
            }),
            (Some(field), None) => changes.push(IdentityChange::FieldRemoved {
                type_id,
                field_id,
                canonical_name: field.canonical_name.clone(),
                virtual_field_index: field.virtual_field_index,
            }),
            (Some(previous), Some(current)) => {
                if previous.canonical_name != current.canonical_name {
                    changes.push(IdentityChange::FieldRenamed {
                        type_id,
                        field_id,
                        from: previous.canonical_name.clone(),
                        to: current.canonical_name.clone(),
                    });
                }
                if previous.virtual_field_index != current.virtual_field_index {
                    changes.push(IdentityChange::FieldReordered {
                        type_id,
                        field_id,
                        from: previous.virtual_field_index,
                        to: current.virtual_field_index,
                    });
                }
                if !previous.canonical_content_eq(current) {
                    changes.push(IdentityChange::FieldChanged { type_id, field_id });
                }
            }
            (None, None) => unreachable!(),
        }
    }
}

fn validate_field_manifest(identity: &TypeIdentity, diagnostics: &mut Vec<ContractDiagnostic>) {
    let mut names = BTreeSet::new();
    let mut ids = BTreeSet::new();
    let mut slots = BTreeSet::new();
    for field in &identity.fields {
        let slot_conflicts = field
            .virtual_field_index
            .is_some_and(|slot| !slots.insert(slot));
        if field.field_id == 0 || !ids.insert(field.field_id) || slot_conflicts {
            diagnostics.push(diagnostic(
                "ID009",
                "field ID must be non-zero and field ID and virtual slot must be unique",
            ));
        }
        if field.canonical_name.is_empty() || !names.insert(field.canonical_name.clone()) {
            diagnostics.push(diagnostic(
                "ID010",
                "empty or duplicate field name in identity manifest",
            ));
        }
    }
}

fn diagnostic(code: &str, message: &str) -> ContractDiagnostic {
    ContractDiagnostic {
        code: code.to_owned(),
        message: message.to_owned(),
        span: None,
    }
}

fn diagnostic_with_span(
    code: &str,
    message: &str,
    span: Option<Range<usize>>,
) -> ContractDiagnostic {
    ContractDiagnostic {
        code: code.to_owned(),
        message: message.to_owned(),
        span,
    }
}
