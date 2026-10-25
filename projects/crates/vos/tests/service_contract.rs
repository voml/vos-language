//! `ServiceContract` construction from Oak service projections.

use vos::{
    CanonicalScalar, CanonicalType, FieldIdentity, IDENTITY_MANIFEST_VERSION, IdentityManifest,
    MethodId, MethodIdentityKey, MethodStreamMode, OperationEffect, OperationSemantics,
    ResolvedCanonicalType, ServiceContractDecl, ServiceId, ServiceIdentityKey, ServiceMethodDecl,
    TypeContractKind, TypeIdentity, build_operation_contracts, build_service_contracts, parse_oak,
    resolve_contract, service_contract_fingerprint,
};

const SOURCE: &str = "namespace demo::auth\n\
\n\
class CreateRequest {\n\
    user_name: utf8,\n\
}\n\
\n\
class DeleteRequest {\n\
    id: uuid,\n\
}\n\
\n\
class Identity {\n\
    id: uuid,\n\
}\n\
\n\
query create(request: CreateRequest) -> Identity { Identity }\n\
\n\
query delete(request: DeleteRequest) -> bool { true }\n\
\n\
service IdentityService {\n\
  create(request: CreateRequest) -> Identity\n\
  delete(request: DeleteRequest) -> bool\n\
}\n";

const TWO_SERVICES: &str = "namespace demo::auth\n\
\n\
class CreateRequest {\n\
    user_name: utf8,\n\
}\n\
\n\
class DeleteRequest {\n\
    id: uuid,\n\
}\n\
\n\
class Identity {\n\
    id: uuid,\n\
}\n\
\n\
query create(request: CreateRequest) -> CreateRequest { request }\n\
\n\
query delete(request: DeleteRequest) -> bool { true }\n\
\n\
service IdentityService {\n\
  create(request: CreateRequest) -> CreateRequest\n\
}\n\
\n\
service AuditService {\n\
  audit(request: CreateRequest) -> CreateRequest\n\
}\n";

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
                canonical_path: vec![
                    "demo".to_owned(),
                    "auth".to_owned(),
                    "DeleteRequest".to_owned(),
                ],
                type_id: 2,
                kind: TypeContractKind::Class,
                fields: vec![FieldIdentity {
                    canonical_name: "id".to_owned(),
                    field_id: 11,
                    virtual_field_index: Some(0),
                }],
            },
            TypeIdentity {
                canonical_path: vec!["demo".to_owned(), "auth".to_owned(), "Identity".to_owned()],
                type_id: 3,
                kind: TypeContractKind::Class,
                fields: vec![FieldIdentity {
                    canonical_name: "id".to_owned(),
                    field_id: 12,
                    virtual_field_index: Some(0),
                }],
            },
        ],
    }
}

fn schema_types(source: &str) -> Vec<vos::ResolvedTypeContract> {
    let projection = parse_oak(source).unwrap().project_schema().unwrap();
    resolve_contract(&projection, &manifest()).unwrap().types
}

fn projected_services(source: &str) -> Vec<vos::ServiceProjection> {
    parse_oak(source).unwrap().project_services().unwrap()
}

fn operation_contracts(source: &str) -> Vec<vos::OperationContract> {
    let operations = parse_oak(source).unwrap().project_operations().unwrap();
    let semantics = operations
        .iter()
        .map(|operation| {
            OperationSemantics::new(
                operation.operation_id.clone(),
                OperationEffect::Read,
                vec![],
            )
        })
        .collect::<Vec<_>>();
    build_operation_contracts(&[(0, &operations)], &semantics, &schema_types(source)).unwrap()
}

fn method(
    name: &str,
    request: &[&str],
    response: &[&str],
    stream: MethodStreamMode,
    operations: &[vos::OperationContract],
    operation: &str,
) -> ServiceMethodDecl {
    ServiceMethodDecl {
        name: name.to_owned(),
        request_type: CanonicalType::Named(
            request
                .iter()
                .map(|segment| (*segment).to_owned())
                .collect(),
        ),
        response_type: CanonicalType::Named(
            response
                .iter()
                .map(|segment| (*segment).to_owned())
                .collect(),
        ),
        stream,
        operation_id: operations
            .iter()
            .find(|item| item.name == operation)
            .unwrap()
            .operation_id
            .clone(),
    }
}

fn declaration(
    service: &vos::ServiceProjection,
    operations: &[vos::OperationContract],
) -> ServiceContractDecl {
    ServiceContractDecl {
        service_id: service.service_id.clone(),
        methods: vec![
            method(
                "create",
                &["CreateRequest"],
                &["Identity"],
                MethodStreamMode::Unary,
                operations,
                "create",
            ),
            method(
                "delete",
                &["DeleteRequest"],
                &["bool"],
                MethodStreamMode::Unary,
                operations,
                "delete",
            ),
        ],
    }
}

fn build(
    source: &str,
    declarations: &[ServiceContractDecl],
) -> Result<Vec<vos::ServiceContract>, Vec<vos::ContractDiagnostic>> {
    let services = projected_services(source);
    build_service_contracts(
        &[(0, &services)],
        declarations,
        &operation_contracts(source),
        &schema_types(source),
    )
}

fn codes(diagnostics: &[vos::ContractDiagnostic]) -> Vec<&str> {
    diagnostics.iter().map(|item| item.code.as_str()).collect()
}

#[test]
fn service_contract_binds_methods_to_declared_operations() {
    let operations = operation_contracts(SOURCE);
    let services = projected_services(SOURCE);
    let contracts = build(SOURCE, &[declaration(&services[0], &operations)]).unwrap();

    assert_eq!(contracts.len(), 1);
    let service = &contracts[0];
    assert_eq!(service.name, "IdentityService");
    assert_eq!(
        service.canonical_path,
        vec!["demo", "auth", "IdentityService"]
    );
    assert_eq!(service.service_id, services[0].service_id);
    assert_eq!(service.validate(), Ok(()));
    assert_eq!(service.methods.len(), 2);
    assert!(
        service.attributes.is_empty(),
        "the fixture source declares no service attributes"
    );

    let create = service
        .methods
        .iter()
        .find(|item| item.name == "create")
        .unwrap();
    assert_eq!(
        create.method_id,
        MethodId::derive(&MethodIdentityKey::new(service.service_id.clone(), "create").unwrap())
    );
    assert_eq!(create.stream, MethodStreamMode::Unary);
    assert_eq!(
        create.request_type,
        ResolvedCanonicalType::User {
            path: vec![
                "demo".to_owned(),
                "auth".to_owned(),
                "CreateRequest".to_owned()
            ],
            type_id: 1,
        }
    );
    assert_eq!(
        create.response_type,
        ResolvedCanonicalType::User {
            path: vec!["demo".to_owned(), "auth".to_owned(), "Identity".to_owned()],
            type_id: 3,
        }
    );
    assert_eq!(
        create.operation_id,
        operations
            .iter()
            .find(|item| item.name == "create")
            .unwrap()
            .operation_id
    );
    assert!(create.attributes.is_empty());

    let delete = service
        .methods
        .iter()
        .find(|item| item.name == "delete")
        .unwrap();
    assert_eq!(
        delete.response_type,
        ResolvedCanonicalType::Scalar(CanonicalScalar::Bool)
    );
    assert_eq!(
        delete.operation_id,
        operations
            .iter()
            .find(|item| item.name == "delete")
            .unwrap()
            .operation_id
    );
}

#[test]
fn projected_service_without_declaration_is_rejected() {
    let services = projected_services(SOURCE);
    let diagnostics = build(SOURCE, &[]).unwrap_err();
    assert_eq!(codes(&diagnostics), vec!["SVC001"]);
    assert!(
        diagnostics[0]
            .message
            .contains(services[0].service_id.as_str()),
        "diagnostic must name the undeclared service"
    );
}

#[test]
fn duplicate_declaration_and_unknown_service_are_rejected() {
    let operations = operation_contracts(SOURCE);
    let services = projected_services(SOURCE);
    let declared = declaration(&services[0], &operations);
    let diagnostics = build(SOURCE, &[declared.clone(), declared]).unwrap_err();
    assert_eq!(codes(&diagnostics), vec!["SVC002"]);

    let unknown = ServiceContractDecl {
        service_id: ServiceId::derive(
            &ServiceIdentityKey::new(vec![
                "demo".to_owned(),
                "auth".to_owned(),
                "MissingService".to_owned(),
            ])
            .unwrap(),
        ),
        methods: vec![],
    };
    let diagnostics =
        build(SOURCE, &[declaration(&services[0], &operations), unknown]).unwrap_err();
    assert!(codes(&diagnostics).contains(&"SVC003"));
}

#[test]
fn method_bound_to_unknown_operation_is_rejected() {
    let services = projected_services(SOURCE);
    let operations = operation_contracts(SOURCE);
    let create_only = operations
        .iter()
        .filter(|item| item.name == "create")
        .cloned()
        .collect::<Vec<_>>();
    let diagnostics = build_service_contracts(
        &[(0, &services)],
        &[declaration(&services[0], &operations)],
        &create_only,
        &schema_types(SOURCE),
    )
    .unwrap_err();
    assert_eq!(codes(&diagnostics), vec!["SVC004"]);
}

#[test]
fn duplicate_method_names_and_operation_reuse_are_rejected() {
    let operations = operation_contracts(SOURCE);
    let services = projected_services(SOURCE);

    let mut duplicate_names = declaration(&services[0], &operations);
    duplicate_names.methods[1].name = "create".to_owned();
    duplicate_names.methods[1].request_type = duplicate_names.methods[0].request_type.clone();
    duplicate_names.methods[1].response_type = duplicate_names.methods[0].response_type.clone();
    let diagnostics = build(SOURCE, &[duplicate_names]).unwrap_err();
    assert!(codes(&diagnostics).contains(&"SVC014"));

    let mut reused_operation = declaration(&services[0], &operations);
    reused_operation.methods[1].operation_id = reused_operation.methods[0].operation_id.clone();
    let diagnostics = build(SOURCE, &[reused_operation]).unwrap_err();
    assert_eq!(codes(&diagnostics), vec!["SVC016"]);
}

#[test]
fn operation_bound_by_two_services_is_rejected() {
    let operations = operation_contracts(TWO_SERVICES);
    let services = projected_services(TWO_SERVICES);
    let create = operations
        .iter()
        .find(|item| item.name == "create")
        .unwrap()
        .operation_id
        .clone();
    let declarations = vec![
        ServiceContractDecl {
            service_id: services[0].service_id.clone(),
            methods: vec![method(
                "create",
                &["CreateRequest"],
                &["CreateRequest"],
                MethodStreamMode::Unary,
                &operations,
                "create",
            )],
        },
        ServiceContractDecl {
            service_id: services[1].service_id.clone(),
            methods: vec![ServiceMethodDecl {
                name: "audit".to_owned(),
                request_type: CanonicalType::Named(vec!["CreateRequest".to_owned()]),
                response_type: CanonicalType::Named(vec!["CreateRequest".to_owned()]),
                stream: MethodStreamMode::Unary,
                operation_id: create,
            }],
        },
    ];
    let diagnostics = build(TWO_SERVICES, &declarations).unwrap_err();
    assert_eq!(codes(&diagnostics), vec!["SVC005"]);
}

#[test]
fn service_projected_from_two_units_is_rejected() {
    let services = projected_services(SOURCE);
    let diagnostics = build_service_contracts(
        &[(0, &services), (1, &services)],
        &[declaration(&services[0], &operation_contracts(SOURCE))],
        &operation_contracts(SOURCE),
        &schema_types(SOURCE),
    )
    .unwrap_err();
    assert_eq!(codes(&diagnostics), vec!["SVC006"]);
}

#[test]
fn undeclared_method_types_are_rejected() {
    let operations = operation_contracts(SOURCE);
    let services = projected_services(SOURCE);
    let mut unknown_type = declaration(&services[0], &operations);
    unknown_type.methods[0].request_type = CanonicalType::Named(vec!["Missing".to_owned()]);
    let diagnostics = build(SOURCE, &[unknown_type]).unwrap_err();
    assert!(codes(&diagnostics).contains(&"RES001"));
}

#[test]
fn service_fingerprint_ignores_order_and_provenance() {
    let operations = operation_contracts(SOURCE);
    let services = projected_services(SOURCE);
    let declared = declaration(&services[0], &operations);
    let base = build(SOURCE, std::slice::from_ref(&declared)).unwrap();

    let mut reversed = declared;
    reversed.methods.reverse();
    let moved = build_service_contracts(
        &[(7, &services)],
        &[reversed],
        &operations,
        &schema_types(SOURCE),
    )
    .unwrap();

    assert_eq!(
        service_contract_fingerprint(&base),
        service_contract_fingerprint(&moved)
    );
}

#[test]
fn declaration_must_match_the_source_signature() {
    let operations = operation_contracts(SOURCE);
    let services = projected_services(SOURCE);

    let mut missing_name = declaration(&services[0], &operations);
    missing_name.methods[0].name = "missing".to_owned();
    assert_eq!(
        codes(&build(SOURCE, &[missing_name]).unwrap_err()),
        vec!["SVC018"]
    );

    let mut undeclared = declaration(&services[0], &operations);
    undeclared.methods.retain(|method| method.name != "delete");
    assert_eq!(
        codes(&build(SOURCE, &[undeclared]).unwrap_err()),
        vec!["SVC019"]
    );

    let mut wrong_request = declaration(&services[0], &operations);
    wrong_request.methods[0].request_type = CanonicalType::Named(vec!["Identity".to_owned()]);
    assert_eq!(
        codes(&build(SOURCE, &[wrong_request]).unwrap_err()),
        vec!["SVC021"]
    );

    let mut wrong_response = declaration(&services[0], &operations);
    wrong_response.methods[0].response_type =
        CanonicalType::Named(vec!["DeleteRequest".to_owned()]);
    assert_eq!(
        codes(&build(SOURCE, &[wrong_response]).unwrap_err()),
        vec!["SVC022"]
    );

    let mut wrong_stream = declaration(&services[0], &operations);
    wrong_stream.methods[0].stream = MethodStreamMode::ServerStream;
    assert_eq!(
        codes(&build(SOURCE, &[wrong_stream]).unwrap_err()),
        vec!["SVC023"]
    );
}

#[test]
fn multi_parameter_methods_are_not_synthesised() {
    let source = "namespace demo::auth\n\
                  \n\
                  class CreateRequest {\n\
                  \x20   user_name: utf8,\n\
                  }\n\
                  \n\
                  class DeleteRequest {\n\
                  \x20   id: uuid,\n\
                  }\n\
                  \n\
                  class Identity {\n\
                  \x20   id: uuid,\n\
                  }\n\
                  \n\
                  query create(request: CreateRequest, extra: CreateRequest) -> CreateRequest { request }\n\
                  \n\
                  service IdentityService {\n\
                  \x20 create(request: CreateRequest, extra: CreateRequest) -> CreateRequest\n\
                  }\n";
    let operations = operation_contracts(source);
    let services = projected_services(source);
    let declared = ServiceContractDecl {
        service_id: services[0].service_id.clone(),
        methods: vec![method(
            "create",
            &["CreateRequest"],
            &["CreateRequest"],
            MethodStreamMode::Unary,
            &operations,
            "create",
        )],
    };
    let diagnostics = build(source, &[declared]).unwrap_err();
    assert_eq!(codes(&diagnostics), vec!["SVC020"]);
}

#[test]
fn service_fingerprint_tracks_binding_semantics() {
    let operations = operation_contracts(SOURCE);
    let services = projected_services(SOURCE);
    let base = build(SOURCE, &[declaration(&services[0], &operations)]).unwrap();
    let base_fingerprint = service_contract_fingerprint(&base);

    let single_source = |return_type: &str| {
        format!(
            "namespace demo::auth\n\
             \n\
             class CreateRequest {{\n\
             \x20   user_name: utf8,\n\
             }}\n\
             \n\
             class DeleteRequest {{\n\
             \x20   id: uuid,\n\
             }}\n\
             \n\
             class Identity {{\n\
             \x20   id: uuid,\n\
             }}\n\
             \n\
             query create(request: CreateRequest) -> CreateRequest {{ request }}\n\
             \n\
             service IdentityService {{\n\
             \x20 create(request: CreateRequest) -> {return_type}\n\
             }}\n"
        )
    };
    let build_single = |return_type: &str, stream: MethodStreamMode| {
        let source = single_source(return_type);
        let services = projected_services(&source);
        let operations = operation_contracts(&source);
        build(
            &source,
            &[ServiceContractDecl {
                service_id: services[0].service_id.clone(),
                methods: vec![method(
                    "create",
                    &["CreateRequest"],
                    &["CreateRequest"],
                    stream,
                    &operations,
                    "create",
                )],
            }],
        )
        .unwrap()
    };
    let unary = build_single("CreateRequest", MethodStreamMode::Unary);
    let streamed = build_single("stream<CreateRequest>", MethodStreamMode::ServerStream);
    assert_ne!(
        service_contract_fingerprint(&unary),
        service_contract_fingerprint(&streamed)
    );

    let mut rebound = base.clone();
    rebound[0].methods[1].operation_id = rebound[0].methods[0].operation_id.clone();
    assert_ne!(base_fingerprint, service_contract_fingerprint(&rebound));
}
