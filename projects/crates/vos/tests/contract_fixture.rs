//! Cross-host contract fixture: Oak source to a sealed `/v0` contract artifact.

use vos::{
    CanonicalType, ContractArtifact, ContractFormat, ContractPayload, ContractProducer,
    FieldIdentity, IDENTITY_MANIFEST_VERSION, IdentityManifest, MethodStreamMode,
    OperationContract, OperationEffect, OperationId, OperationSemantics, ResolvedTypeContract,
    ServiceContractDecl, ServiceMethodDecl, ServiceProjection, TypeContractKind, TypeIdentity,
    build_operation_contracts, build_service_contracts, parse_oak, resolve_contract,
};

const SOURCE: &str =
    include_str!("../../../../documentation/fixtures/contracts/operations_services.vos");
const GOLDEN: &str =
    include_str!("../../../../documentation/fixtures/contracts/operations_services.contract.json");
const HERMES_GOLDEN: &str = include_str!(
    "../../../../documentation/fixtures/contracts/operations_services.hermes.contract.json"
);

fn manifest() -> IdentityManifest {
    IdentityManifest {
        format_version: IDENTITY_MANIFEST_VERSION.to_owned(),
        types: vec![
            TypeIdentity {
                canonical_path: vec![
                    "demo".to_owned(),
                    "auth".to_owned(),
                    "CreateRequest".to_owned(),
                ],
                type_id: 1,
                kind: TypeContractKind::Class,
                fields: vec![FieldIdentity {
                    canonical_name: "user_name".to_owned(),
                    field_id: 10,
                    virtual_field_index: Some(0),
                }],
            },
            TypeIdentity {
                canonical_path: vec!["demo".to_owned(), "auth".to_owned(), "Identity".to_owned()],
                type_id: 2,
                kind: TypeContractKind::Class,
                fields: vec![FieldIdentity {
                    canonical_name: "id".to_owned(),
                    field_id: 11,
                    virtual_field_index: Some(0),
                }],
            },
        ],
    }
}

fn schema_types() -> Vec<ResolvedTypeContract> {
    let input = parse_oak(SOURCE).unwrap();
    resolve_contract(&input.project_schema().unwrap(), &manifest())
        .unwrap()
        .types
}

fn operation_contracts() -> Vec<OperationContract> {
    let input = parse_oak(SOURCE).unwrap();
    let operations = input.project_operations().unwrap();
    let semantics = operations
        .iter()
        .map(|operation| {
            let effect = if operation.name == "create" {
                OperationEffect::Write
            } else {
                OperationEffect::Read
            };
            OperationSemantics::new(operation.operation_id.clone(), effect, vec![])
        })
        .collect::<Vec<_>>();
    build_operation_contracts(&[(0, &operations)], &semantics, &schema_types()).unwrap()
}

fn operation_id(operations: &[OperationContract], name: &str) -> OperationId {
    operations
        .iter()
        .find(|item| item.name == name)
        .unwrap()
        .operation_id
        .clone()
}

fn declaration(
    service: &ServiceProjection,
    operations: &[OperationContract],
) -> ServiceContractDecl {
    ServiceContractDecl {
        service_id: service.service_id.clone(),
        methods: vec![
            ServiceMethodDecl {
                name: "create".to_owned(),
                request_type: CanonicalType::Named(vec!["CreateRequest".to_owned()]),
                response_type: CanonicalType::Named(vec!["Identity".to_owned()]),
                stream: MethodStreamMode::Unary,
                operation_id: operation_id(operations, "create"),
            },
            ServiceMethodDecl {
                name: "lookup".to_owned(),
                request_type: CanonicalType::Named(vec!["uuid".to_owned()]),
                response_type: CanonicalType::Optional(Box::new(CanonicalType::Named(vec![
                    "Identity".to_owned(),
                ]))),
                stream: MethodStreamMode::ServerStream,
                operation_id: operation_id(operations, "lookup"),
            },
        ],
    }
}

fn artifact() -> ContractArtifact {
    let input = parse_oak(SOURCE).unwrap();
    let services = input.project_services().unwrap();
    let operations = operation_contracts();
    let types = schema_types();
    let service_contracts = build_service_contracts(
        &[(0, &services)],
        &[declaration(&services[0], &operations)],
        &operations,
        &types,
    )
    .unwrap();

    ContractArtifact::seal(
        ContractFormat::Iris,
        ContractProducer::new("fixture", "0"),
        ContractPayload {
            operations,
            services: service_contracts,
        },
    )
}

#[test]
fn contract_fixture_matches_reviewed_golden() {
    let artifact = artifact();
    let actual: serde_json::Value = serde_json::from_str(&artifact.to_json().unwrap()).unwrap();
    let expected: serde_json::Value = serde_json::from_str(GOLDEN).unwrap();
    assert_eq!(actual, expected);
    assert_eq!(ContractArtifact::from_json(GOLDEN).unwrap(), artifact);
}

#[test]
fn fixture_covers_unary_and_server_stream_methods() {
    let artifact = artifact();
    let methods = &artifact.payload.services[0].methods;
    assert!(
        methods
            .iter()
            .any(|method| method.stream == MethodStreamMode::Unary),
        "the fixture must exercise unary methods"
    );
    assert!(
        methods
            .iter()
            .any(|method| method.stream == MethodStreamMode::ServerStream),
        "the fixture must exercise server-stream methods"
    );
}

#[test]
fn hermes_golden_seals_the_iris_payload_under_its_own_format() {
    let iris = artifact();
    let hermes = ContractArtifact::from_json(HERMES_GOLDEN).unwrap();

    assert_eq!(iris.version, ContractFormat::Iris);
    assert_eq!(hermes.version, ContractFormat::Hermes);
    assert_ne!(
        iris.producer, hermes.producer,
        "producer is provenance only, never part of the contract"
    );
    assert_eq!(iris.fingerprint, hermes.fingerprint);
    assert_eq!(iris.payload, hermes.payload);
    assert_eq!(
        hermes,
        ContractArtifact::seal(
            ContractFormat::Hermes,
            hermes.producer.clone(),
            hermes.payload.clone()
        ),
        "the hermes golden must be a real seal of the shared payload"
    );
    assert_eq!(
        hermes.to_json().unwrap().trim_end(),
        HERMES_GOLDEN.trim_end(),
        "the hermes golden must be the canonical encoding of the sealed artifact"
    );
}

#[test]
fn method_bound_to_unknown_operation_is_reported() {
    let input = parse_oak(SOURCE).unwrap();
    let services = input.project_services().unwrap();
    let operations = operation_contracts();
    let types = schema_types();
    let missing = OperationId::parse(&format!("op_{}", "0".repeat(64))).unwrap();

    let mut declared = declaration(&services[0], &operations);
    declared.methods[0].operation_id = missing.clone();
    let diagnostics =
        build_service_contracts(&[(0, &services)], &[declared], &operations, &types).unwrap_err();

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "SVC004");
    assert!(
        diagnostics[0].message.contains(missing.as_str()),
        "diagnostic must name the missing operation"
    );
}
