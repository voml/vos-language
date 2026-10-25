//! Negative conformance for the sealed cross-host contract fixtures.
//!
//! Every case mutates a reviewed golden and asserts the exact diagnostic code,
//! so a host sees a stable rejection instead of a panic or a silent acceptance.

use serde_json::Value;
use vos::{
    ContractArtifact, ContractAttribute, ContractFormat, ContractPayload, ContractProducer,
    MethodId, OperationId,
};

const IRIS_GOLDEN: &str =
    include_str!("../../../../documentation/fixtures/contracts/operations_services.contract.json");
const HERMES_GOLDEN: &str = include_str!(
    "../../../../documentation/fixtures/contracts/operations_services.hermes.contract.json"
);

fn payload(golden: &str) -> ContractPayload {
    let wire: Value = serde_json::from_str(golden).unwrap();
    serde_json::from_value(wire["payload"].clone()).unwrap()
}

fn seal(payload: ContractPayload) -> ContractArtifact {
    ContractArtifact::seal(
        ContractFormat::Iris,
        ContractProducer::new("fixture", "0"),
        payload,
    )
}

#[test]
fn duplicate_method_names_are_rejected() {
    let mut payload = payload(IRIS_GOLDEN);
    let name = payload.services[0].methods[0].name.clone();
    payload.services[0].methods[1].name = name;

    assert_eq!(seal(payload).validate().unwrap_err().code, "SVC014");
}

#[test]
fn duplicate_method_ids_are_rejected() {
    let mut payload = payload(IRIS_GOLDEN);
    let method_id = payload.services[0].methods[0].method_id.clone();
    payload.services[0].methods[1].method_id = method_id;

    assert_eq!(seal(payload).validate().unwrap_err().code, "SVC015");
}

#[test]
fn method_id_must_match_its_name_and_service() {
    let mut payload = payload(IRIS_GOLDEN);
    payload.services[0].methods[1].method_id =
        MethodId::parse(&format!("mtd_{}", "1".repeat(64))).unwrap();

    assert_eq!(seal(payload).validate().unwrap_err().code, "SVC015");
}

#[test]
fn malformed_method_ids_are_rejected_without_panicking() {
    for golden in [IRIS_GOLDEN, HERMES_GOLDEN] {
        let mut wire: Value = serde_json::from_str(golden).unwrap();
        wire["payload"]["services"][0]["methods"][0]["methodId"] =
            Value::String("mtd_deadbeef".to_owned());
        let payload: ContractPayload = serde_json::from_value(wire["payload"].clone()).unwrap();

        assert_eq!(seal(payload).validate().unwrap_err().code, "SVC015");
    }
}

#[test]
fn unknown_fields_are_rejected() {
    let mut method: Value = serde_json::from_str(IRIS_GOLDEN).unwrap();
    method["payload"]["services"][0]["methods"][0]["handler"] =
        Value::String("identity.create".to_owned());
    assert_eq!(
        ContractArtifact::from_json(&method.to_string())
            .unwrap_err()
            .code,
        "ART010"
    );

    let mut root: Value = serde_json::from_str(IRIS_GOLDEN).unwrap();
    root["unexpected"] = Value::Bool(true);
    assert_eq!(
        ContractArtifact::from_json(&root.to_string())
            .unwrap_err()
            .code,
        "ART010"
    );
}

#[test]
fn stale_fingerprints_are_rejected() {
    let mut wire: Value = serde_json::from_str(IRIS_GOLDEN).unwrap();
    wire["payload"]["operations"][0]["effect"] = Value::String("read".to_owned());
    assert_eq!(
        ContractArtifact::from_json(&wire.to_string())
            .unwrap_err()
            .code,
        "ART020"
    );

    let mut hermes = ContractArtifact::from_json(HERMES_GOLDEN).unwrap();
    hermes.fingerprint = "0".repeat(64);
    assert_eq!(hermes.validate().unwrap_err().code, "ART020");
}

#[test]
fn reviewed_fixtures_still_validate_with_their_golden_fingerprints() {
    for golden in [IRIS_GOLDEN, HERMES_GOLDEN] {
        let wire: Value = serde_json::from_str(golden).unwrap();
        let artifact = ContractArtifact::from_json(golden).unwrap();

        assert_eq!(artifact.validate(), Ok(()));
        assert_eq!(artifact.fingerprint, wire["fingerprint"]);
        assert_eq!(seal(payload(golden)).fingerprint, artifact.fingerprint);
    }
}

#[test]
fn method_bound_to_an_operation_outside_the_payload_is_rejected() {
    for golden in [IRIS_GOLDEN, HERMES_GOLDEN] {
        let mut payload = payload(golden);
        payload.services[0].methods[0].operation_id =
            OperationId::parse(&format!("op_{}", "0".repeat(64))).unwrap();

        assert_eq!(seal(payload).validate().unwrap_err().code, "SVC004");
    }
}

#[test]
fn duplicate_operation_ids_are_rejected() {
    let mut payload = payload(IRIS_GOLDEN);
    let duplicate = payload.operations[0].clone();
    payload.operations.push(duplicate);

    assert_eq!(seal(payload).validate().unwrap_err().code, "OPC003");
}

#[test]
fn duplicate_services_are_rejected() {
    let mut payload = payload(IRIS_GOLDEN);
    let duplicate = payload.services[0].clone();
    payload.services.push(duplicate);

    assert_eq!(seal(payload).validate().unwrap_err().code, "SVC006");
}

#[test]
fn unknown_attributes_round_trip_through_the_sealed_artifact() {
    let base = seal(payload(IRIS_GOLDEN));
    let mut payload = payload(IRIS_GOLDEN);
    payload.services[0].attributes = vec![
        ContractAttribute {
            name: "route".to_owned(),
            syntax: "@route(\"/rpc\")".to_owned(),
        },
        ContractAttribute {
            name: "authorize".to_owned(),
            syntax: "@authorize(\"admin\")".to_owned(),
        },
    ];
    payload.services[0].methods[0].attributes = vec![ContractAttribute {
        name: "wire".to_owned(),
        syntax: "@wire(\"json\")".to_owned(),
    }];

    let artifact = seal(payload.clone());
    assert_ne!(
        artifact.fingerprint, base.fingerprint,
        "attributes must enter the payload fingerprint"
    );
    assert_eq!(artifact.validate(), Ok(()));
    assert_eq!(
        artifact.payload.services[0].methods[0].method_id,
        base.payload.services[0].methods[0].method_id,
        "attributes must never change the method identity"
    );
    assert_eq!(
        artifact.payload.services[0].methods[0].operation_id,
        base.payload.services[0].methods[0].operation_id,
        "attributes must never change the operation binding"
    );

    let decoded = ContractArtifact::from_json(&artifact.to_json().unwrap()).unwrap();
    assert_eq!(decoded, artifact);
    assert_eq!(
        decoded.payload.services[0].attributes,
        payload.services[0].attributes
    );
    assert_eq!(
        decoded.payload.services[0].methods[0].attributes,
        payload.services[0].methods[0].attributes
    );
}
