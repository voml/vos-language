use vos_types::{
    CanonicalScalar, ContractArtifact, ContractFormat, ContractPayload, ContractProducer, MethodId,
    MethodIdentityKey, MethodStreamMode, OperationCardinality, OperationContract, OperationEffect,
    OperationId, OperationIdentityKey, OperationKind, ResolvedCanonicalType, ServiceContract,
    ServiceId, ServiceIdentityKey, ServiceMethodContract, contract_payload_fingerprint,
};

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

fn create_operation() -> OperationContract {
    OperationContract {
        operation_id: operation_id("create"),
        kind: OperationKind::Query,
        name: "create".to_owned(),
        canonical_path: vec!["demo".to_owned(), "create".to_owned()],
        parameters: vec![],
        result_type: Some(ResolvedCanonicalType::Scalar(CanonicalScalar::Bool)),
        cardinality: OperationCardinality::One,
        effect: OperationEffect::Write,
        capabilities: vec![],
        source_unit_id: 0,
        span: (0..0).into(),
        stable_id_hint: None,
    }
}

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

fn service() -> ServiceContract {
    service_named("IdentityService")
}

fn service_named(name: &str) -> ServiceContract {
    let service_id = ServiceId::derive(
        &ServiceIdentityKey::new(vec!["demo".to_owned(), "auth".to_owned(), name.to_owned()])
            .unwrap(),
    );
    ServiceContract {
        service_id: service_id.clone(),
        name: name.to_owned(),
        canonical_path: vec!["demo".to_owned(), "auth".to_owned(), name.to_owned()],
        methods: vec![ServiceMethodContract {
            method_id: MethodId::derive(&MethodIdentityKey::new(service_id, "create").unwrap()),
            name: "create".to_owned(),
            request_type: ResolvedCanonicalType::Scalar(CanonicalScalar::Uuid),
            response_type: ResolvedCanonicalType::Scalar(CanonicalScalar::Bool),
            stream: MethodStreamMode::Unary,
            operation_id: operation_id("create"),
            attributes: vec![],
            stable_id_hint: None,
        }],
        attributes: vec![],
        source_unit_id: 0,
        span: (0..40).into(),
        stable_id_hint: None,
    }
}

fn payload() -> ContractPayload {
    ContractPayload {
        operations: vec![create_operation()],
        services: vec![service()],
    }
}

fn artifact() -> ContractArtifact {
    ContractArtifact::seal(
        ContractFormat::Iris,
        ContractProducer::new("fixture", "0"),
        payload(),
    )
}

#[test]
fn artifact_round_trips_with_stable_field_names() {
    let artifact = artifact();
    let wire = serde_json::to_value(&artifact).unwrap();

    assert_eq!(wire["version"], "vos/iris.contract/v0");
    assert_eq!(wire["producer"]["name"], "fixture");
    assert_eq!(wire["producer"]["version"], "0");
    assert_eq!(wire["fingerprint"], artifact.fingerprint);
    assert_eq!(
        wire["payload"]["operations"][0]["operationId"],
        operation_id("create").as_str()
    );
    assert_eq!(
        wire["payload"]["services"][0]["serviceId"],
        service_id().as_str()
    );
    assert_eq!(
        wire["payload"]["services"][0]["methods"][0]["methodId"],
        service().methods[0].method_id.as_str()
    );

    let json = artifact.to_json().unwrap();
    let decoded = ContractArtifact::from_json(&json).unwrap();
    assert_eq!(decoded, artifact);
    assert_eq!(decoded.validate(), Ok(()));
}

#[test]
fn old_and_unknown_versions_are_rejected() {
    let wire = serde_json::to_value(artifact()).unwrap();
    for version in [
        "vos/iris.contract/v1",
        "vos/iris.contract",
        "vos/hades.contract/v0",
        "",
    ] {
        let mut tampered = wire.clone();
        tampered["version"] = serde_json::Value::String(version.to_owned());
        let error = ContractArtifact::from_json(&tampered.to_string()).unwrap_err();
        assert_eq!(error.code, "ART010", "version {version} must be rejected");
    }

    let mut unknown_field = wire;
    unknown_field["terms"] = serde_json::Value::Array(vec![]);
    assert_eq!(
        ContractArtifact::from_json(&unknown_field.to_string())
            .unwrap_err()
            .code,
        "ART010"
    );
}

#[test]
fn stale_fingerprint_is_rejected() {
    let mut stale = artifact();
    stale.fingerprint = "0".repeat(64);
    assert_eq!(stale.validate().unwrap_err().code, "ART020");

    let mut json = serde_json::to_value(artifact()).unwrap();
    json["payload"]["operations"][0]["effect"] = serde_json::Value::String("read".to_owned());
    assert_eq!(
        ContractArtifact::from_json(&json.to_string())
            .unwrap_err()
            .code,
        "ART020"
    );
}

#[test]
fn missing_producer_is_rejected() {
    let mut no_name = artifact();
    no_name.producer = ContractProducer::new("", "0");
    assert_eq!(no_name.validate().unwrap_err().code, "ART021");

    let mut no_version = artifact();
    no_version.producer = ContractProducer::new("fixture", " ");
    assert_eq!(no_version.validate().unwrap_err().code, "ART021");
}

#[test]
fn invalid_payload_keeps_its_contract_diagnostic() {
    let mut payload = payload();
    payload.services[0].methods[0].method_id =
        MethodId::parse(&format!("mtd_{}", "0".repeat(64))).unwrap();
    let artifact = ContractArtifact::seal(ContractFormat::Hermes, ContractProducer::vos(), payload);
    assert_eq!(artifact.validate().unwrap_err().code, "SVC015");
}

#[test]
fn method_bound_to_a_missing_operation_is_rejected_after_reseal() {
    let mut payload = payload();
    payload.services[0].methods[0].operation_id =
        OperationId::parse(&format!("op_{}", "0".repeat(64))).unwrap();

    let artifact = ContractArtifact::seal(
        ContractFormat::Iris,
        ContractProducer::new("fixture", "0"),
        payload,
    );
    assert_eq!(artifact.validate().unwrap_err().code, "SVC004");
}

#[test]
fn duplicate_operation_ids_are_rejected_after_reseal() {
    let mut payload = payload();
    payload.operations.push(payload.operations[0].clone());

    let artifact = ContractArtifact::seal(
        ContractFormat::Iris,
        ContractProducer::new("fixture", "0"),
        payload,
    );
    assert_eq!(artifact.validate().unwrap_err().code, "OPC003");
}

#[test]
fn duplicate_service_identity_is_rejected_after_reseal() {
    let mut payload = payload();
    payload.services.push(payload.services[0].clone());

    let artifact = ContractArtifact::seal(
        ContractFormat::Iris,
        ContractProducer::new("fixture", "0"),
        payload,
    );
    assert_eq!(artifact.validate().unwrap_err().code, "SVC006");
}

#[test]
fn operation_bound_by_two_services_is_rejected_after_reseal() {
    let mut payload = payload();
    payload.services.push(service_named("OtherService"));

    let artifact = ContractArtifact::seal(
        ContractFormat::Iris,
        ContractProducer::new("fixture", "0"),
        payload,
    );
    assert_eq!(artifact.validate().unwrap_err().code, "SVC005");
}

#[test]
fn fingerprint_binds_payload_only() {
    let iris = ContractArtifact::seal(
        ContractFormat::Iris,
        ContractProducer::new("iris-host", "1"),
        payload(),
    );
    let hermes = ContractArtifact::seal(
        ContractFormat::Hermes,
        ContractProducer::new("hermes-host", "9"),
        payload(),
    );
    assert_eq!(iris.fingerprint, hermes.fingerprint);
    assert_eq!(iris.fingerprint, contract_payload_fingerprint(&payload()));
    assert_eq!(iris.fingerprint.len(), 64);

    let mut changed = payload();
    changed.operations[0].effect = OperationEffect::Pure;
    assert_ne!(
        iris.fingerprint,
        ContractArtifact::seal(
            ContractFormat::Iris,
            ContractProducer::new("iris-host", "1"),
            changed
        )
        .fingerprint
    );
}
