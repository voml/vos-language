use vos_types::{
    CanonicalScalar, ContractAttribute, MethodId, MethodIdentityKey, MethodStreamMode, OperationId,
    OperationIdentityKey, OperationKind, ResolvedCanonicalType, ServiceContract, ServiceId,
    ServiceIdentityKey, ServiceMethodContract, service_contract_fingerprint,
};

fn service_id() -> ServiceId {
    ServiceId::derive(
        &ServiceIdentityKey::new(vec![
            "demo".to_owned(),
            "auth".to_owned(),
            "IdentityService".to_owned(),
        ])
        .unwrap(),
    )
}

fn operation_id(name: &str) -> OperationId {
    OperationId::derive(
        &OperationIdentityKey::new(
            &["demo".to_owned()],
            OperationKind::Query,
            name,
            vec![],
            "unit",
        )
        .unwrap(),
    )
}

fn method(
    service: &ServiceId,
    name: &str,
    response: ResolvedCanonicalType,
    stream: MethodStreamMode,
    operation: OperationId,
) -> ServiceMethodContract {
    ServiceMethodContract {
        method_id: MethodId::derive(&MethodIdentityKey::new(service.clone(), name).unwrap()),
        name: name.to_owned(),
        request_type: ResolvedCanonicalType::Scalar(CanonicalScalar::Utf8),
        response_type: response,
        stream,
        operation_id: operation,
        attributes: vec![],
    }
}

fn attribute(name: &str, syntax: &str) -> ContractAttribute {
    ContractAttribute {
        name: name.to_owned(),
        syntax: syntax.to_owned(),
    }
}

fn identity_type() -> ResolvedCanonicalType {
    ResolvedCanonicalType::User {
        path: vec!["demo".to_owned(), "auth".to_owned(), "Identity".to_owned()],
        type_id: 3,
    }
}

fn service() -> ServiceContract {
    let service_id = service_id();
    ServiceContract {
        service_id: service_id.clone(),
        name: "IdentityService".to_owned(),
        canonical_path: vec![
            "demo".to_owned(),
            "auth".to_owned(),
            "IdentityService".to_owned(),
        ],
        methods: vec![
            method(
                &service_id,
                "create",
                identity_type(),
                MethodStreamMode::Unary,
                operation_id("create"),
            ),
            method(
                &service_id,
                "delete",
                ResolvedCanonicalType::Scalar(CanonicalScalar::Bool),
                MethodStreamMode::ServerStream,
                operation_id("delete"),
            ),
        ],
        attributes: vec![],
        source_unit_id: 0,
        span: (0..40).into(),
    }
}

#[test]
fn service_contract_round_trips_with_stable_field_names() {
    let contract = service();
    let wire = serde_json::to_value(&contract).unwrap();

    assert_eq!(wire["serviceId"], contract.service_id.as_str());
    assert_eq!(wire["name"], "IdentityService");
    assert_eq!(wire["canonicalPath"][2], "IdentityService");
    assert_eq!(
        wire["methods"][0]["methodId"],
        contract.methods[0].method_id.as_str()
    );
    assert_eq!(wire["methods"][0]["name"], "create");
    assert_eq!(wire["methods"][0]["requestType"]["scalar"], "utf8");
    assert_eq!(wire["methods"][0]["responseType"]["user"]["typeId"], 3);
    assert_eq!(wire["methods"][0]["stream"], "unary");
    assert_eq!(
        wire["methods"][0]["operationId"],
        contract.methods[0].operation_id.as_str()
    );
    assert_eq!(wire["methods"][1]["stream"], "server-stream");
    assert_eq!(wire["attributes"], serde_json::json!([]));
    assert_eq!(wire["methods"][0]["attributes"], serde_json::json!([]));
    assert_eq!(wire["sourceUnitId"], 0);
    assert_eq!(wire["span"]["start"], 0);
    assert_eq!(wire["span"]["end"], 40);

    let decoded: ServiceContract = serde_json::from_value(wire).unwrap();
    assert_eq!(decoded, contract);
    assert_eq!(decoded.validate(), Ok(()));
}

#[test]
fn unknown_fields_and_missing_fields_are_rejected() {
    let mut wire = serde_json::to_value(service()).unwrap();
    wire["body"] = serde_json::Value::String("create(request)".to_owned());
    assert!(
        serde_json::from_value::<ServiceContract>(wire.clone()).is_err(),
        "unknown contract fields must not be accepted"
    );

    let mut unknown_method = serde_json::to_value(service()).unwrap();
    unknown_method["methods"][0]["handler"] =
        serde_json::Value::String("identity.create".to_owned());
    assert!(serde_json::from_value::<ServiceContract>(unknown_method).is_err());

    let mut missing = serde_json::to_value(service()).unwrap();
    missing.as_object_mut().unwrap().remove("methods");
    assert!(serde_json::from_value::<ServiceContract>(missing).is_err());

    let mut missing_stream = serde_json::to_value(service()).unwrap();
    missing_stream["methods"][0]
        .as_object_mut()
        .unwrap()
        .remove("stream");
    assert!(serde_json::from_value::<ServiceContract>(missing_stream).is_err());

    let mut missing_attributes = serde_json::to_value(service()).unwrap();
    missing_attributes
        .as_object_mut()
        .unwrap()
        .remove("attributes");
    assert!(
        serde_json::from_value::<ServiceContract>(missing_attributes).is_err(),
        "service attributes must be declared explicitly, not defaulted"
    );

    let mut missing_method_attributes = serde_json::to_value(service()).unwrap();
    missing_method_attributes["methods"][0]
        .as_object_mut()
        .unwrap()
        .remove("attributes");
    assert!(serde_json::from_value::<ServiceContract>(missing_method_attributes).is_err());
}

#[test]
fn validation_rejects_inconsistent_contracts() {
    let mut malformed = serde_json::to_value(service()).unwrap();
    malformed["serviceId"] = serde_json::Value::String(String::new());
    let malformed: ServiceContract = serde_json::from_value(malformed).unwrap();
    assert_eq!(malformed.validate().unwrap_err().code, "SVC010");

    let mut path = service();
    path.canonical_path = vec!["demo".to_owned(), "other".to_owned()];
    assert_eq!(path.validate().unwrap_err().code, "SVC011");

    let mut no_methods = service();
    no_methods.methods.clear();
    assert_eq!(no_methods.validate().unwrap_err().code, "SVC012");

    let mut mismatched_id = service();
    mismatched_id.service_id = ServiceId::derive(
        &ServiceIdentityKey::new(vec![
            "demo".to_owned(),
            "auth".to_owned(),
            "Other".to_owned(),
        ])
        .unwrap(),
    );
    assert_eq!(mismatched_id.validate().unwrap_err().code, "SVC013");

    let mut blank_method = service();
    blank_method.methods[0].name = String::new();
    assert_eq!(blank_method.validate().unwrap_err().code, "SVC014");

    let mut duplicate_method = service();
    duplicate_method.methods[1].name = "create".to_owned();
    duplicate_method.methods[1].method_id = MethodId::derive(
        &MethodIdentityKey::new(duplicate_method.service_id.clone(), "create").unwrap(),
    );
    assert_eq!(duplicate_method.validate().unwrap_err().code, "SVC014");

    let mut fabricated_method = service();
    fabricated_method.methods[0].method_id =
        MethodId::parse(&format!("mtd_{}", "0".repeat(64))).unwrap();
    assert_eq!(fabricated_method.validate().unwrap_err().code, "SVC015");

    let mut rebound = service();
    rebound.methods[1].operation_id = rebound.methods[0].operation_id.clone();
    assert_eq!(rebound.validate().unwrap_err().code, "SVC016");

    let mut implicit_request = service();
    implicit_request.methods[0].request_type = ResolvedCanonicalType::User {
        path: vec!["demo".to_owned(), "auth".to_owned(), "Identity".to_owned()],
        type_id: 0,
    };
    assert_eq!(implicit_request.validate().unwrap_err().code, "SVC017");
}

#[test]
fn fingerprint_ignores_provenance_and_method_order() {
    let base = service_contract_fingerprint(&[service()]);

    let mut reordered = service();
    reordered.methods.reverse();
    reordered.source_unit_id = 9;
    reordered.span = (120..200).into();

    assert_eq!(base.len(), 64);
    assert_eq!(base, service_contract_fingerprint(&[reordered]));
}

#[test]
fn fingerprint_changes_with_binding_semantics() {
    let base = service_contract_fingerprint(&[service()]);

    let mut streamed = service();
    streamed.methods[0].stream = MethodStreamMode::ServerStream;
    assert_ne!(base, service_contract_fingerprint(&[streamed]));

    let mut retyped = service();
    retyped.methods[0].response_type = ResolvedCanonicalType::Scalar(CanonicalScalar::Utf8);
    assert_ne!(base, service_contract_fingerprint(&[retyped]));

    let mut rebound = service();
    rebound.methods[1].operation_id = operation_id("create");
    assert_ne!(base, service_contract_fingerprint(&[rebound]));

    let mut renamed = service();
    renamed.name = "AuthService".to_owned();
    renamed.canonical_path = vec![
        "demo".to_owned(),
        "auth".to_owned(),
        "AuthService".to_owned(),
    ];
    assert_ne!(base, service_contract_fingerprint(&[renamed]));
}

#[test]
fn unknown_attributes_are_kept_and_do_not_change_the_signature() {
    let mut contract = service();
    let method_id = contract.methods[0].method_id.clone();
    let operation_id = contract.methods[0].operation_id.clone();
    contract.attributes = vec![
        attribute("route", "@route(\"/rpc\")"),
        attribute("authorize", "@authorize(\"admin\")"),
    ];
    contract.methods[0].attributes = vec![attribute("wire", "@wire(\"json\")")];

    let wire = serde_json::to_value(&contract).unwrap();
    assert_eq!(wire["attributes"][0]["name"], "route");
    assert_eq!(wire["attributes"][0]["syntax"], "@route(\"/rpc\")");
    assert_eq!(wire["attributes"][1]["name"], "authorize");
    assert_eq!(wire["methods"][0]["attributes"][0]["name"], "wire");
    assert_eq!(
        wire["methods"][0]["attributes"][0]["syntax"],
        "@wire(\"json\")"
    );

    let decoded: ServiceContract = serde_json::from_value(wire).unwrap();
    assert_eq!(decoded, contract);
    assert_eq!(decoded.validate(), Ok(()));
    assert_eq!(decoded.methods[0].method_id, method_id);
    assert_eq!(decoded.methods[0].operation_id, operation_id);
}

#[test]
fn duplicate_attributes_are_rejected_in_their_own_scope() {
    let mut duplicate_service = service();
    duplicate_service.attributes = vec![
        attribute("route", "@route(\"/rpc\")"),
        attribute("route", "@route(\"/rpc\")"),
    ];
    assert_eq!(
        duplicate_service.validate().unwrap_err().code,
        "SVC007",
        "service attribute duplicates use the service scope code"
    );

    let mut blank_service = service();
    blank_service.attributes = vec![attribute(" ", "@blank")];
    assert_eq!(blank_service.validate().unwrap_err().code, "SVC007");

    let mut duplicate_method = service();
    duplicate_method.methods[0].attributes = vec![
        attribute("wire", "@wire(\"json\")"),
        attribute("wire", "@wire(\"json\")"),
    ];
    assert_eq!(
        duplicate_method.validate().unwrap_err().code,
        "SVC008",
        "method attribute duplicates use the method scope code"
    );

    let mut blank_method = service();
    blank_method.methods[1].attributes = vec![attribute("", "@blank")];
    assert_eq!(blank_method.validate().unwrap_err().code, "SVC008");
}

#[test]
fn attributes_with_the_same_name_and_different_syntax_coexist() {
    let mut contract = service();
    contract.attributes = vec![
        attribute("route", "@route(\"/rpc\")"),
        attribute("route", "@route(\"/http\")"),
    ];
    contract.methods[0].attributes = vec![
        attribute("wire", "@wire(\"json\")"),
        attribute("wire", "@wire(\"binary\")"),
    ];
    assert_eq!(contract.validate(), Ok(()));
}

#[test]
fn fingerprint_tracks_attributes_but_not_spans() {
    let base = service_contract_fingerprint(&[service()]);

    let mut tagged = service();
    tagged.attributes = vec![attribute("route", "@route(\"/rpc\")")];
    assert_ne!(base, service_contract_fingerprint(&[tagged.clone()]));

    let mut method_tagged = service();
    method_tagged.methods[0].attributes = vec![attribute("wire", "@wire(\"json\")")];
    assert_ne!(base, service_contract_fingerprint(&[method_tagged]));

    let mut respanned = tagged.clone();
    respanned.source_unit_id = 4;
    respanned.span = (500..900).into();
    assert_eq!(
        service_contract_fingerprint(&[tagged]),
        service_contract_fingerprint(&[respanned])
    );
}
