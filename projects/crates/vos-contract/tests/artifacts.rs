use serde_json::{Value, json};
use vos_contract::{ContractDiagnostic, SchemaProjection, parse_oak};

#[test]
fn projection_artifact_matches_reviewed_golden() {
    let source = include_str!("../../../../specifications/fixtures/contracts/projection_basic.vos");
    let golden = include_str!("../../../../specifications/fixtures/contracts/projection_basic.contract.json");
    let input = parse_oak(source).expect("Oak parses fixture");
    let contract = input.project_schema().expect("VOS projects supported fixture");
    let actual: Value = serde_json::from_str(&contract.to_json().unwrap()).unwrap();
    let expected: Value = serde_json::from_str(golden).unwrap();
    assert_eq!(actual, expected);
    assert_eq!(SchemaProjection::from_json(golden).unwrap(), contract);
    assert_eq!(contract.to_json().unwrap(), contract.to_json().unwrap());
}

#[test]
fn strict_reader_rejects_unknown_fields_versions_and_invalid_spans() {
    let golden = include_str!("../../../../specifications/fixtures/contracts/projection_basic.contract.json");

    let mut unknown: Value = serde_json::from_str(golden).unwrap();
    unknown["envelope"]["unexpected"] = json!(true);
    let error = SchemaProjection::from_json(&serde_json::to_string(&unknown).unwrap()).unwrap_err();
    assert_eq!(error.code, "ART001");

    let mut wrong_stage: Value = serde_json::from_str(golden).unwrap();
    wrong_stage["envelope"]["stage"] = json!("resolved-contract");
    let error = SchemaProjection::from_json(&serde_json::to_string(&wrong_stage).unwrap()).unwrap_err();
    assert_eq!(error.code, "ART002");

    let mut wrong_source_unit: Value = serde_json::from_str(golden).unwrap();
    wrong_source_unit["envelope"]["sourceUnits"][0]["sourceUnitId"] = json!(1);
    let error = SchemaProjection::from_json(&serde_json::to_string(&wrong_source_unit).unwrap()).unwrap_err();
    assert_eq!(error.code, "ART003");

    let mut wrong_span: Value = serde_json::from_str(golden).unwrap();
    wrong_span["types"][0]["fields"][0]["span"]["end"] = json!(999);
    let error = SchemaProjection::from_json(&serde_json::to_string(&wrong_span).unwrap()).unwrap_err();
    assert_eq!(error.code, "ART004");
}

#[test]
fn diagnostics_use_byte_span_objects_including_null() {
    let source = "class 用户 { 名字: uuid 名字: utf8 }";
    let diagnostics = parse_oak(source).unwrap().project_schema().unwrap_err();
    assert_eq!(diagnostics.len(), 1);
    let start = source.rfind("名字").unwrap();
    let actual = serde_json::to_value(&diagnostics[0]).unwrap();
    assert_eq!(actual, json!({
        "code": "VOS003",
        "message": "duplicate field declaration",
        "span": {"start": start, "end": start + "名字".len()}
    }));
    let global = ContractDiagnostic { code: "test".to_owned(), message: "global".to_owned(), span: None };
    assert_eq!(serde_json::to_value(global).unwrap()["span"], Value::Null);
}

#[test]
fn unsupported_declarations_never_disappear_from_a_successful_contract() {
    for declaration in [
        "using shared::User;",
        "enums State { Active = 1 }",
        "flags Scope { Read = 1 }",
        "obsolete table Old;",
        "const NAME: utf8 = \"value\";",
        "service Api {}",
        "query users() {}",
        "udf value() {}",
        "micro value() {}",
    ] {
        let source = format!("class T {{ id: uuid }}\n{declaration}");
        let input = parse_oak(&source).unwrap();
        let diagnostics = input.project_schema().expect_err("unsupported declaration cannot be dropped");
        assert_eq!(diagnostics[0].code, "VOS004", "{source}");
    }
}

#[test]
fn attribute_groups_and_arguments_require_oak_structure() {
    for attribute in ["[primary, wire]", "[backfill(\"value\")]", "[]"] {
        let source = format!("class T {{ {attribute} id: uuid }}");
        let diagnostics = parse_oak(&source).unwrap().project_schema().unwrap_err();
        assert_eq!(diagnostics[0].code, "VOS005");
        assert_eq!(&source[diagnostics[0].span.clone().unwrap()], attribute);
    }
}
