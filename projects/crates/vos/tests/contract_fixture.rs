//! Cross-host contract fixture: Oak source to a sealed `/v0` contract artifact.

use vos::{
    CanonicalType, ContractArtifact, ContractFormat, ContractPayload, ContractProducer,
    FieldIdentity, IDENTITY_MANIFEST_VERSION, IdentityManifest, MethodStreamMode, OperationEffect,
    OperationSemantics, ServiceContractDecl, ServiceMethodDecl, TypeContractKind, TypeIdentity,
    build_operation_contracts, build_service_contracts, parse_oak, resolve_contract,
};

const SOURCE: &str =
    include_str!("../../../../documentation/fixtures/contracts/operations_services.vos");
const GOLDEN: &str =
    include_str!("../../../../documentation/fixtures/contracts/operations_services.contract.json");

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

fn artifact() -> ContractArtifact {
    let input = parse_oak(SOURCE).unwrap();
    let types = resolve_contract(&input.project_schema().unwrap(), &manifest())
        .unwrap()
        .types;
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
    let operation_contracts =
        build_operation_contracts(&[(0, &operations)], &semantics, &types).unwrap();

    let services = input.project_services().unwrap();
    let declaration = ServiceContractDecl {
        service_id: services[0].service_id.clone(),
        methods: vec![
            ServiceMethodDecl {
                name: "create".to_owned(),
                request_type: CanonicalType::Named(vec!["CreateRequest".to_owned()]),
                response_type: CanonicalType::Named(vec!["Identity".to_owned()]),
                stream: MethodStreamMode::Unary,
                operation_id: operation_contracts
                    .iter()
                    .find(|item| item.name == "create")
                    .unwrap()
                    .operation_id
                    .clone(),
            },
            ServiceMethodDecl {
                name: "lookup".to_owned(),
                request_type: CanonicalType::Named(vec!["uuid".to_owned()]),
                response_type: CanonicalType::Optional(Box::new(CanonicalType::Named(vec![
                    "Identity".to_owned(),
                ]))),
                stream: MethodStreamMode::ServerStream,
                operation_id: operation_contracts
                    .iter()
                    .find(|item| item.name == "lookup")
                    .unwrap()
                    .operation_id
                    .clone(),
            },
        ],
    };
    let service_contracts = build_service_contracts(
        &[(0, &services)],
        &[declaration],
        &operation_contracts,
        &types,
    )
    .unwrap();

    ContractArtifact::seal(
        ContractFormat::Iris,
        ContractProducer::new("fixture", "0"),
        ContractPayload {
            operations: operation_contracts,
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
