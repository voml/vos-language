use vos_types::{
    CONTRACT_IDENTITY_VERSION, ID_HEX_LEN, METHOD_ID_PREFIX, MethodId, MethodIdentityKey,
    OPERATION_ID_PREFIX, OperationId, OperationIdentityKey, OperationKind, OperationParameterKey,
    SERVICE_ID_PREFIX, ServiceId, ServiceIdentityKey,
};

fn operation_key(name: &str, parameters: &[(&str, &str)], result: &str) -> OperationIdentityKey {
    let parameters = parameters
        .iter()
        .map(|(parameter, type_key)| {
            OperationParameterKey::new(parameter, type_key).expect("valid parameter key")
        })
        .collect();
    OperationIdentityKey::new(OperationKind::Macro, name, parameters, result)
        .expect("valid operation key")
}

fn service_key(segments: &[&str]) -> ServiceIdentityKey {
    ServiceIdentityKey::new(segments.iter().map(|item| (*item).to_owned()).collect())
        .expect("valid service key")
}

#[test]
fn operation_id_wire_form_is_prefix_and_lowercase_hex() {
    let key = operation_key("public_name", &[("value", "utf8")], "utf8");
    let id = OperationId::derive(&key);
    let wire = id.as_str();

    assert!(wire.starts_with(OPERATION_ID_PREFIX));
    assert_eq!(wire.len(), OPERATION_ID_PREFIX.len() + ID_HEX_LEN);
    assert!(
        wire[OPERATION_ID_PREFIX.len()..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    );
    assert_eq!(OperationId::parse(wire).expect("round trip"), id);
}

#[test]
fn identity_derivation_is_repeated_build_stable() {
    let first = OperationId::derive(&operation_key("active", &[("limit", "i64")], "[User]"));
    let second = OperationId::derive(&operation_key("active", &[("limit", "i64")], "[User]"));
    let service = ServiceId::derive(&service_key(&["demo", "auth", "IdentityService"]));
    let again = ServiceId::derive(&service_key(&["demo", "auth", "IdentityService"]));
    let method = MethodId::derive(&MethodIdentityKey::new(service.clone(), "create").unwrap());
    let method_again = MethodId::derive(&MethodIdentityKey::new(again.clone(), "create").unwrap());

    assert_eq!(first, second);
    assert_eq!(service, again);
    assert_eq!(method, method_again);
}

#[test]
fn renaming_changes_operation_and_service_identities() {
    let base = OperationId::derive(&operation_key("active", &[("limit", "i64")], "[User]"));
    let renamed = OperationId::derive(&operation_key("activated", &[("limit", "i64")], "[User]"));
    assert_ne!(base, renamed);

    let service = ServiceId::derive(&service_key(&["demo", "auth", "IdentityService"]));
    let moved = ServiceId::derive(&service_key(&["demo", "auth", "SessionService"]));
    let nested = ServiceId::derive(&service_key(&[
        "demo",
        "auth",
        "identity",
        "IdentityService",
    ]));
    assert_ne!(service, moved);
    assert_ne!(service, nested);

    let method = MethodId::derive(&MethodIdentityKey::new(service.clone(), "create").unwrap());
    let method_renamed = MethodId::derive(&MethodIdentityKey::new(service, "delete").unwrap());
    assert_ne!(method, method_renamed);
}

#[test]
fn parameter_names_types_and_order_are_semantic() {
    let base = OperationId::derive(&operation_key(
        "active",
        &[("status", "utf8"), ("limit", "i64")],
        "[User]",
    ));
    let renamed = OperationId::derive(&operation_key(
        "active",
        &[("state", "utf8"), ("limit", "i64")],
        "[User]",
    ));
    let retyped = OperationId::derive(&operation_key(
        "active",
        &[("status", "uuid"), ("limit", "i64")],
        "[User]",
    ));
    let reordered = OperationId::derive(&operation_key(
        "active",
        &[("limit", "i64"), ("status", "utf8")],
        "[User]",
    ));

    assert_ne!(base, renamed);
    assert_ne!(base, retyped);
    assert_ne!(base, reordered);
    assert_ne!(renamed, retyped);
}

#[test]
fn declaration_kind_participates_in_operation_identity() {
    let parameters = vec![OperationParameterKey::new("value", "utf8").unwrap()];
    let macro_key = OperationIdentityKey::new(
        OperationKind::Macro,
        "public_name",
        parameters.clone(),
        "utf8",
    )
    .unwrap();
    let query_key =
        OperationIdentityKey::new(OperationKind::Query, "public_name", parameters, "utf8").unwrap();

    assert_eq!(macro_key.identity_version, CONTRACT_IDENTITY_VERSION);
    assert_ne!(
        OperationId::derive(&macro_key),
        OperationId::derive(&query_key)
    );
}

#[test]
fn malformed_identities_are_rejected() {
    let operation = OperationId::derive(&operation_key("active", &[], "unit"));
    let wire = operation.as_str();

    assert_eq!(OperationId::parse("").unwrap_err().code, "ID030");
    assert_eq!(
        OperationId::parse(&wire[OPERATION_ID_PREFIX.len() + 1..])
            .unwrap_err()
            .code,
        "ID030"
    );
    assert_eq!(
        OperationId::parse(&wire.to_uppercase()).unwrap_err().code,
        "ID030"
    );
    assert_eq!(
        OperationId::parse(&wire[..wire.len() - 1])
            .unwrap_err()
            .code,
        "ID030"
    );
    assert_eq!(
        ServiceId::parse(&format!("{SERVICE_ID_PREFIX}{}", "0".repeat(ID_HEX_LEN)))
            .expect("valid service id")
            .as_str()
            .len(),
        SERVICE_ID_PREFIX.len() + ID_HEX_LEN
    );
    assert_eq!(ServiceId::parse(wire).unwrap_err().code, "ID031");
    assert_eq!(
        MethodId::parse(&format!("{METHOD_ID_PREFIX}{}", "a".repeat(ID_HEX_LEN)))
            .expect("valid method id")
            .as_str()
            .len(),
        METHOD_ID_PREFIX.len() + ID_HEX_LEN
    );
    assert_eq!(
        MethodId::parse(&operation.to_string()).unwrap_err().code,
        "ID032"
    );
}

#[test]
fn invalid_identity_keys_are_rejected_instead_of_canonicalized() {
    assert_eq!(
        OperationIdentityKey::new(OperationKind::Query, "", Vec::new(), "unit")
            .unwrap_err()
            .code,
        "ID033"
    );
    assert_eq!(
        OperationIdentityKey::new(OperationKind::Query, "has space", Vec::new(), "unit")
            .unwrap_err()
            .code,
        "ID033"
    );
    assert_eq!(
        OperationParameterKey::new("", "utf8").unwrap_err().code,
        "ID033"
    );
    assert_eq!(
        OperationParameterKey::new("value", "").unwrap_err().code,
        "ID037"
    );
    let duplicate = vec![
        OperationParameterKey::new("value", "utf8").unwrap(),
        OperationParameterKey::new("value", "i64").unwrap(),
    ];
    assert_eq!(
        OperationIdentityKey::new(OperationKind::Udf, "double", duplicate, "unit")
            .unwrap_err()
            .code,
        "ID034"
    );
    assert_eq!(
        ServiceIdentityKey::new(Vec::new()).unwrap_err().code,
        "ID035"
    );
    assert_eq!(
        ServiceIdentityKey::new(vec!["demo".to_owned(), String::new()])
            .unwrap_err()
            .code,
        "ID035"
    );
    let service = ServiceId::derive(&service_key(&["demo", "IdentityService"]));
    assert_eq!(
        MethodIdentityKey::new(service, "").unwrap_err().code,
        "ID036"
    );
}

#[test]
fn identity_keys_serialize_without_source_text() {
    let key = operation_key("active", &[("limit", "i64")], "[User]");
    let json = serde_json::to_value(&key).unwrap();

    assert_eq!(json["identityVersion"], CONTRACT_IDENTITY_VERSION);
    assert_eq!(json["kind"], "macro");
    assert_eq!(json["name"], "active");
    assert_eq!(json["parameters"][0]["name"], "limit");
    assert_eq!(json["parameters"][0]["typeKey"], "i64");
    assert_eq!(json["result"], "[User]");
    assert!(json.get("span").is_none());
    assert!(json.get("source").is_none());
}
