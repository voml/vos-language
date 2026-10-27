use vos_types::{
    CanonicalScalar, OperationCardinality, OperationContract, OperationEffect, OperationId,
    OperationIdentityKey, OperationKind, OperationParameterContract, OperationParameterKey,
    ResolvedCanonicalType, SemanticCapability, operation_contract_fingerprint,
};

fn operation(
    name: &str,
    effect: OperationEffect,
    capabilities: Vec<SemanticCapability>,
    parameters: &[(&str, CanonicalScalar)],
    result: Option<ResolvedCanonicalType>,
    result_key: &str,
) -> OperationContract {
    let parameters = parameters
        .iter()
        .map(|(parameter, scalar)| OperationParameterContract {
            name: (*parameter).to_owned(),
            canonical_type: ResolvedCanonicalType::Scalar(*scalar),
        })
        .collect::<Vec<_>>();
    let identity_parameters = parameters
        .iter()
        .map(|parameter| {
            OperationParameterKey::new(
                &parameter.name,
                &canonical_type_key(&parameter.canonical_type),
            )
            .unwrap()
        })
        .collect();
    let identity = OperationIdentityKey::new(
        &[],
        OperationKind::Query,
        name,
        identity_parameters,
        result_key,
    )
    .unwrap();
    let cardinality = OperationContract::cardinality_of(&result);
    OperationContract {
        operation_id: OperationId::derive(&identity),
        kind: OperationKind::Query,
        name: name.to_owned(),
        canonical_path: vec![name.to_owned()],
        parameters,
        result_type: result,
        cardinality,
        effect,
        capabilities,
        source_unit_id: 0,
        span: (0..0).into(),
        stable_id_hint: None,
    }
}

fn canonical_type_key(canonical_type: &ResolvedCanonicalType) -> String {
    match canonical_type {
        ResolvedCanonicalType::Scalar(scalar) => scalar.as_str().to_owned(),
        _ => "other".to_owned(),
    }
}

fn active() -> OperationContract {
    operation(
        "active",
        OperationEffect::Read,
        vec![SemanticCapability::Storage],
        &[
            ("status", CanonicalScalar::Utf8),
            ("limit", CanonicalScalar::I64),
        ],
        Some(ResolvedCanonicalType::List(Box::new(
            ResolvedCanonicalType::Scalar(CanonicalScalar::Uuid),
        ))),
        "[uuid]",
    )
}

fn latest_audit() -> OperationContract {
    operation(
        "latest_audit",
        OperationEffect::Pure,
        vec![SemanticCapability::Aggregation, SemanticCapability::Storage],
        &[("at", CanonicalScalar::DateTime)],
        None,
        "unit",
    )
}

#[test]
fn operation_contract_round_trips_with_stable_field_names() {
    let contract = active();
    let wire = serde_json::to_value(&contract).unwrap();

    assert_eq!(wire["operationId"], contract.operation_id.as_str());
    assert_eq!(wire["kind"], "query");
    assert_eq!(wire["name"], "active");
    assert_eq!(wire["canonicalPath"][0], "active");
    assert_eq!(wire["parameters"][0]["name"], "status");
    assert_eq!(wire["parameters"][0]["canonicalType"]["scalar"], "utf8");
    assert_eq!(wire["cardinality"], "many");
    assert_eq!(wire["effect"], "read");
    assert_eq!(wire["capabilities"][0], "storage");
    assert_eq!(wire["sourceUnitId"], 0);
    assert_eq!(wire["span"]["start"], 0);
    assert_eq!(wire["span"]["end"], 0);

    let decoded: OperationContract = serde_json::from_value(wire).unwrap();
    assert_eq!(decoded, contract);
    assert_eq!(decoded.validate(), Ok(()));
}

#[test]
fn unknown_fields_and_missing_fields_are_rejected() {
    let mut wire = serde_json::to_value(active()).unwrap();
    wire["body"] = serde_json::Value::String("User.filter".to_owned());
    assert!(
        serde_json::from_value::<OperationContract>(wire.clone()).is_err(),
        "unknown contract fields must not be accepted"
    );

    let mut missing = serde_json::to_value(active()).unwrap();
    missing.as_object_mut().unwrap().remove("cardinality");
    assert!(serde_json::from_value::<OperationContract>(missing).is_err());
}

#[test]
fn fingerprint_ignores_provenance_and_declaration_order() {
    let first = active();
    let second = latest_audit();
    let mut reordered = second.clone();
    reordered.source_unit_id = 7;
    reordered.span = (100..140).into();
    let mut moved_first = first.clone();
    moved_first.source_unit_id = 3;
    moved_first.span = (5..60).into();

    let base = operation_contract_fingerprint(&[first.clone(), second.clone()]);
    let swapped = operation_contract_fingerprint(&[reordered.clone(), moved_first.clone()]);
    let repeated = operation_contract_fingerprint(&[first, second]);

    assert_eq!(base.len(), 64);
    assert_eq!(base, swapped);
    assert_eq!(base, repeated);
}

#[test]
fn fingerprint_changes_with_semantics() {
    let base = operation_contract_fingerprint(&[active()]);

    let mut renamed = active();
    renamed.name = "activated".to_owned();
    renamed.canonical_path = vec!["activated".to_owned()];
    assert_ne!(base, operation_contract_fingerprint(&[renamed]));

    let mut write = active();
    write.effect = OperationEffect::Write;
    assert_ne!(base, operation_contract_fingerprint(&[write]));

    let mut retyped = active();
    retyped.parameters[1].canonical_type = ResolvedCanonicalType::Scalar(CanonicalScalar::U32);
    assert_ne!(base, operation_contract_fingerprint(&[retyped]));

    let mut list_result = active();
    list_result.result_type = Some(ResolvedCanonicalType::Scalar(CanonicalScalar::Uuid));
    list_result.cardinality = OperationCardinality::One;
    assert_ne!(base, operation_contract_fingerprint(&[list_result]));

    let mut searched = active();
    searched.capabilities.push(SemanticCapability::Search);
    assert_ne!(base, operation_contract_fingerprint(&[searched]));
}

#[test]
fn fingerprint_ignores_capability_order() {
    let sorted = operation(
        "audit",
        OperationEffect::Read,
        vec![SemanticCapability::Aggregation, SemanticCapability::Storage],
        &[],
        None,
        "unit",
    );
    let reversed = operation(
        "audit",
        OperationEffect::Read,
        vec![SemanticCapability::Storage, SemanticCapability::Aggregation],
        &[],
        None,
        "unit",
    );

    assert_eq!(
        operation_contract_fingerprint(&[sorted]),
        operation_contract_fingerprint(&[reversed])
    );
}

#[test]
fn validation_rejects_inconsistent_contracts() {
    let mut cardinality = active();
    cardinality.cardinality = OperationCardinality::One;
    assert_eq!(cardinality.validate().unwrap_err().code, "OPC013");

    let mut unsorted = active();
    unsorted.capabilities = vec![SemanticCapability::Storage, SemanticCapability::Aggregation];
    assert_eq!(unsorted.validate().unwrap_err().code, "OPC014");

    let mut duplicate = active();
    duplicate.capabilities = vec![SemanticCapability::Storage, SemanticCapability::Storage];
    assert_eq!(duplicate.validate().unwrap_err().code, "OPC014");

    let mut empty_parameter = active();
    empty_parameter.parameters[0].name = String::new();
    assert_eq!(empty_parameter.validate().unwrap_err().code, "OPC012");

    let mut duplicate_parameter = active();
    duplicate_parameter.parameters[1].name = "status".to_owned();
    assert_eq!(duplicate_parameter.validate().unwrap_err().code, "OPC012");

    let mut path = active();
    path.canonical_path = vec!["demo".to_owned(), "other".to_owned()];
    assert_eq!(path.validate().unwrap_err().code, "OPC011");

    let mut malformed = serde_json::to_value(active()).unwrap();
    malformed["operationId"] = serde_json::Value::String("op_deadbeef".to_owned());
    let malformed: OperationContract = serde_json::from_value(malformed).unwrap();
    assert_eq!(malformed.validate().unwrap_err().code, "OPC010");
}
