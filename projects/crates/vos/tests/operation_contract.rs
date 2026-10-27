//! `OperationContract` construction from Oak projections.

use vos::{
    CanonicalScalar, FieldIdentity, IDENTITY_MANIFEST_VERSION, IdentityManifest,
    OperationCardinality, OperationContract, OperationEffect, OperationSemantics,
    ResolvedCanonicalType, SemanticCapability, TypeContractKind, TypeIdentity,
    build_operation_contracts, operation_contract_fingerprint, parse_oak, resolve_contract,
};

fn operations(source: &str) -> Vec<vos::types::OperationProjection> {
    parse_oak(source).unwrap().project_operations().unwrap()
}

fn manifest() -> IdentityManifest {
    IdentityManifest {
        format_version: IDENTITY_MANIFEST_VERSION.to_owned(),
        types: vec![TypeIdentity {
            canonical_path: vec!["demo".to_owned(), "auth".to_owned(), "User".to_owned()],
            type_id: 1,
            kind: TypeContractKind::Class,
            fields: vec![FieldIdentity {
                canonical_name: "id".to_owned(),
                field_id: 2,
                virtual_field_index: Some(0),
            }],
        }],
    }
}

fn contract_types(source: &str) -> Vec<vos::ResolvedTypeContract> {
    let input = parse_oak(source).unwrap();
    resolve_contract(&input.project_schema().unwrap(), &manifest())
        .unwrap()
        .types
}

fn semantics_for(
    operations: &[vos::types::OperationProjection],
    effect: OperationEffect,
    capabilities: Vec<SemanticCapability>,
) -> Vec<OperationSemantics> {
    operations
        .iter()
        .map(|operation| {
            OperationSemantics::new(operation.operation_id.clone(), effect, capabilities.clone())
        })
        .collect()
}

const SERVICE_SOURCE: &str = "namespace demo::auth\n\
\n\
class User {\n\
    id: uuid,\n\
}\n\
\n\
query active(status: utf8, limit: i64) -> [User] { User }\n\
\n\
macro as_of(at: DateTime<UTC>, amount: decimal) -> d128 { amount }\n";

#[test]
fn canonical_types_are_resolved_for_parameters_and_results() {
    let operations = operations(SERVICE_SOURCE);
    let semantics = semantics_for(&operations, OperationEffect::Read, vec![]);
    let contracts = build_operation_contracts(
        &[(0, &operations)],
        &semantics,
        &contract_types(SERVICE_SOURCE),
        None,
    )
    .unwrap();

    let active = contracts
        .iter()
        .find(|contract| contract.name == "active")
        .unwrap();
    assert_eq!(
        active.parameters[0].canonical_type,
        ResolvedCanonicalType::Scalar(CanonicalScalar::Utf8)
    );
    assert_eq!(
        active.parameters[1].canonical_type,
        ResolvedCanonicalType::Scalar(CanonicalScalar::I64)
    );
    assert_eq!(active.cardinality, OperationCardinality::Many);
    assert_eq!(
        active.result_type,
        Some(ResolvedCanonicalType::List(Box::new(
            ResolvedCanonicalType::User {
                path: vec!["demo".to_owned(), "auth".to_owned(), "User".to_owned()],
                type_id: 1,
            }
        )))
    );

    let as_of = contracts
        .iter()
        .find(|contract| contract.name == "as_of")
        .unwrap();
    assert_eq!(as_of.canonical_path, vec!["demo", "auth", "as_of"]);
    assert_eq!(
        as_of.parameters[0].canonical_type,
        ResolvedCanonicalType::Scalar(CanonicalScalar::DateTime)
    );
    assert_eq!(
        as_of.parameters[1].canonical_type,
        ResolvedCanonicalType::Scalar(CanonicalScalar::Decimal)
    );
    assert_eq!(
        as_of.result_type,
        Some(ResolvedCanonicalType::Scalar(CanonicalScalar::Decimal))
    );
    assert_eq!(as_of.cardinality, OperationCardinality::One);
    assert_eq!(as_of.effect, OperationEffect::Read);
}

#[test]
fn unknown_parameter_types_are_rejected_instead_of_defaulting() {
    let source = "query active(status: Missing) -> [User] { User }";
    let operations = operations(source);
    let semantics = semantics_for(&operations, OperationEffect::Read, vec![]);

    let diagnostics = build_operation_contracts(&[(0, &operations)], &semantics, &[], None).unwrap_err();
    assert!(
        diagnostics.iter().any(|item| item.code == "RES001"),
        "unresolved types must surface as diagnostics, got {diagnostics:?}"
    );
}

#[test]
fn missing_declared_effect_is_rejected() {
    let operations = operations(SERVICE_SOURCE);
    let diagnostics =
        build_operation_contracts(&[(0, &operations)], &[], &contract_types(SERVICE_SOURCE), None)
            .unwrap_err();

    assert_eq!(diagnostics.len(), 2);
    assert!(
        diagnostics.iter().all(|item| item.code == "OPC001"),
        "every operation needs declared semantics, got {diagnostics:?}"
    );
}

#[test]
fn duplicate_and_unknown_semantics_are_rejected() {
    let projections = operations(SERVICE_SOURCE);
    let types = contract_types(SERVICE_SOURCE);
    let mut semantics = semantics_for(&projections, OperationEffect::Pure, vec![]);
    let duplicated = semantics[0].clone();
    semantics.push(duplicated);
    let diagnostics =
        build_operation_contracts(&[(0, &projections)], &semantics, &types, None).unwrap_err();
    assert!(diagnostics.iter().any(|item| item.code == "OPC002"));

    let mut unknown = semantics_for(&projections, OperationEffect::Pure, vec![]);
    let stray = operations(&SERVICE_SOURCE.replace("as_of", "as_of_renamed"));
    unknown.push(OperationSemantics::new(
        stray[1].operation_id.clone(),
        OperationEffect::Pure,
        vec![],
    ));
    let diagnostics =
        build_operation_contracts(&[(0, &projections)], &unknown, &types, None).unwrap_err();
    assert!(diagnostics.iter().any(|item| item.code == "OPC004"));
}

#[test]
fn the_same_operation_in_two_units_is_rejected() {
    let operations = operations(SERVICE_SOURCE);
    let semantics = semantics_for(&operations, OperationEffect::Read, vec![]);
    let diagnostics = build_operation_contracts(
        &[(0, &operations), (1, &operations)],
        &semantics,
        &contract_types(SERVICE_SOURCE),
        None,
    )
    .unwrap_err();

    assert_eq!(
        diagnostics
            .iter()
            .filter(|item| item.code == "OPC003")
            .count(),
        2
    );
}

#[test]
fn contracts_are_order_independent_across_source_units() {
    let operations = operations(SERVICE_SOURCE);
    let semantics = semantics_for(
        &operations,
        OperationEffect::Read,
        vec![SemanticCapability::Storage],
    );
    let types = contract_types(SERVICE_SOURCE);
    let forward = build_operation_contracts(
        &[(0, &operations[..1]), (1, &operations[1..])],
        &semantics,
        &types,
        None,
    )
    .unwrap();
    let reverse = build_operation_contracts(
        &[(9, &operations[1..]), (4, &operations[..1])],
        &semantics,
        &types,
        None,
    )
    .unwrap();

    assert_eq!(
        forward
            .iter()
            .map(|contract| contract.operation_id.clone())
            .collect::<Vec<_>>(),
        reverse
            .iter()
            .map(|contract| contract.operation_id.clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        forward
            .iter()
            .find(|contract| contract.name == "active")
            .unwrap()
            .source_unit_id,
        0
    );
    assert_eq!(
        reverse
            .iter()
            .find(|contract| contract.name == "active")
            .unwrap()
            .source_unit_id,
        4
    );
    assert_eq!(
        operation_contract_fingerprint(&forward),
        operation_contract_fingerprint(&reverse)
    );
    assert_eq!(
        forward
            .iter()
            .find(|contract| contract.name == "active")
            .unwrap()
            .capabilities,
        vec![SemanticCapability::Storage]
    );
}

#[test]
fn declared_effects_are_carried_into_the_contract() {
    let operations = operations(SERVICE_SOURCE);
    let types = contract_types(SERVICE_SOURCE);
    let semantics = vec![
        OperationSemantics::new(
            operations[0].operation_id.clone(),
            OperationEffect::Read,
            vec![SemanticCapability::Storage],
        ),
        OperationSemantics::new(
            operations[1].operation_id.clone(),
            OperationEffect::Pure,
            vec![],
        ),
    ];
    let contracts = build_operation_contracts(&[(0, &operations)], &semantics, &types, None).unwrap();

    let active = contracts
        .iter()
        .find(|contract| contract.name == "active")
        .unwrap();
    let as_of = contracts
        .iter()
        .find(|contract| contract.name == "as_of")
        .unwrap();
    assert_eq!(active.effect, OperationEffect::Read);
    assert_eq!(as_of.effect, OperationEffect::Pure);
    assert_eq!(active.kind, vos::OperationKind::Query);
    assert_eq!(as_of.kind, vos::OperationKind::Macro);
    assert_eq!(
        contracts
            .iter()
            .map(|contract: &OperationContract| contract.operation_id.clone())
            .collect::<Vec<_>>(),
        {
            let mut ids = contracts
                .iter()
                .map(|contract| contract.operation_id.clone())
                .collect::<Vec<_>>();
            ids.sort();
            ids
        }
    );
}
