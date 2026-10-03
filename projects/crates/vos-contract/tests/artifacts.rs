use serde_json::{Value, json};
use vos_contract::{
    ContractDiagnostic, FieldIdentity, IDENTITY_MANIFEST_VERSION, IdentityManifest,
    SchemaProjection, TypeContractKind, TypeIdentity, bind_identity, parse_oak,
    schema_fingerprint,
};

#[test]
fn projection_artifact_matches_reviewed_golden() {
    let source = include_str!("../../../../specifications/fixtures/contracts/projection_basic.vos");
    let golden = include_str!(
        "../../../../specifications/fixtures/contracts/projection_basic.contract.json"
    );
    let input = parse_oak(source).expect("Oak parses fixture");
    let contract = input
        .project_schema()
        .expect("VOS projects supported fixture");
    let actual: Value = serde_json::from_str(&contract.to_json().unwrap()).unwrap();
    let expected: Value = serde_json::from_str(golden).unwrap();
    assert_eq!(actual, expected);
    assert_eq!(SchemaProjection::from_json(golden).unwrap(), contract);
    assert_eq!(contract.to_json().unwrap(), contract.to_json().unwrap());
}

#[test]
fn strict_reader_rejects_unknown_fields_versions_and_invalid_spans() {
    let golden = include_str!(
        "../../../../specifications/fixtures/contracts/projection_basic.contract.json"
    );

    let mut unknown: Value = serde_json::from_str(golden).unwrap();
    unknown["envelope"]["unexpected"] = json!(true);
    let error = SchemaProjection::from_json(&serde_json::to_string(&unknown).unwrap()).unwrap_err();
    assert_eq!(error.code, "ART001");

    let mut wrong_stage: Value = serde_json::from_str(golden).unwrap();
    wrong_stage["envelope"]["stage"] = json!("resolved-contract");
    let error =
        SchemaProjection::from_json(&serde_json::to_string(&wrong_stage).unwrap()).unwrap_err();
    assert_eq!(error.code, "ART002");

    let mut wrong_source_unit: Value = serde_json::from_str(golden).unwrap();
    wrong_source_unit["envelope"]["sourceUnits"][0]["sourceUnitId"] = json!(1);
    let error = SchemaProjection::from_json(&serde_json::to_string(&wrong_source_unit).unwrap())
        .unwrap_err();
    assert_eq!(error.code, "ART003");

    let mut wrong_span: Value = serde_json::from_str(golden).unwrap();
    wrong_span["types"][0]["fields"][0]["span"]["end"] = json!(999);
    let error =
        SchemaProjection::from_json(&serde_json::to_string(&wrong_span).unwrap()).unwrap_err();
    assert_eq!(error.code, "ART004");
}

#[test]
fn diagnostics_use_byte_span_objects_including_null() {
    let source = "class 用户 { 名字: uuid 名字: utf8 }";
    let diagnostics = parse_oak(source).unwrap().project_schema().unwrap_err();
    assert_eq!(diagnostics.len(), 1);
    let start = source.rfind("名字").unwrap();
    let actual = serde_json::to_value(&diagnostics[0]).unwrap();
    assert_eq!(
        actual,
        json!({
            "code": "VOS003",
            "message": "duplicate field declaration",
            "span": {"start": start, "end": start + "名字".len()}
        })
    );
    let global = ContractDiagnostic {
        code: "test".to_owned(),
        message: "global".to_owned(),
        span: None,
    };
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
        let diagnostics = input
            .project_schema()
            .expect_err("unsupported declaration cannot be dropped");
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

#[test]
fn feature_matrix_keeps_projection_separate_from_resolution_and_consumption() {
    let matrix: Value = serde_json::from_str(include_str!(
        "../../../../specifications/fixtures/contracts/feature-matrix.json"
    ))
    .unwrap();
    assert_eq!(matrix["formatVersion"], "vos-feature-matrix-v0");
    let features = matrix["features"].as_array().unwrap();
    let identity = features
        .iter()
        .find(|feature| feature["id"] == "durable-identity.fingerprint")
        .unwrap();
    assert_eq!(identity["projected"], false);
    assert_eq!(identity["downstream-consumed"], false);
    let fields = features
        .iter()
        .find(|feature| feature["id"] == "table-class.fields")
        .unwrap();
    assert_eq!(fields["projected"], true);
    assert_eq!(fields["resolved"], "partial");
}

#[test]
fn identity_manifest_binding_ignores_manifest_and_source_order() {
    let input = parse_oak("class B { b: utf8, a: uuid }\nclass A { id: uuid }").unwrap();
    let projection = input.project_schema().unwrap();
    let manifest = IdentityManifest {
        format_version: IDENTITY_MANIFEST_VERSION.to_owned(),
        types: vec![
            TypeIdentity {
                canonical_path: vec!["A".to_owned()],
                type_id: 7,
                kind: TypeContractKind::Class,
                fields: vec![FieldIdentity {
                    canonical_name: "id".to_owned(),
                    field_id: 10,
                    virtual_field_index: 0,
                }],
            },
            TypeIdentity {
                canonical_path: vec!["B".to_owned()],
                type_id: 3,
                kind: TypeContractKind::Class,
                fields: vec![
                    FieldIdentity {
                        canonical_name: "a".to_owned(),
                        field_id: 12,
                        virtual_field_index: 1,
                    },
                    FieldIdentity {
                        canonical_name: "b".to_owned(),
                        field_id: 11,
                        virtual_field_index: 0,
                    },
                ],
            },
        ],
    };
    let bound = bind_identity(&projection, &manifest).unwrap();
    assert_eq!(bound.types[0].canonical_path, ["B"]);
    assert_eq!(bound.types[0].type_id, 3);
    assert_eq!(bound.types[0].fields[0].canonical_name, "b");
    assert_eq!(bound.types[0].fields[0].field_id, 11);
    assert_eq!(bound.types[0].fields[1].field_id, 12);
    assert_eq!(bound.types[1].type_id, 7);
}

#[test]
fn identity_manifest_rejects_missing_unknown_duplicate_and_zero_ids() {
    let projection = parse_oak("class A { id: uuid }")
        .unwrap()
        .project_schema()
        .unwrap();
    let cases = [
        IdentityManifest {
            format_version: "wrong".to_owned(),
            types: vec![],
        },
        IdentityManifest {
            format_version: IDENTITY_MANIFEST_VERSION.to_owned(),
            types: vec![],
        },
        IdentityManifest {
            format_version: IDENTITY_MANIFEST_VERSION.to_owned(),
            types: vec![TypeIdentity {
                canonical_path: vec!["Unknown".to_owned()],
                type_id: 1,
                kind: TypeContractKind::Class,
                fields: vec![],
            }],
        },
        IdentityManifest {
            format_version: IDENTITY_MANIFEST_VERSION.to_owned(),
            types: vec![TypeIdentity {
                canonical_path: vec!["A".to_owned()],
                type_id: 0,
                kind: TypeContractKind::Class,
                fields: vec![FieldIdentity {
                    canonical_name: "id".to_owned(),
                    field_id: 1,
                    virtual_field_index: 0,
                }],
            }],
        },
    ];
    for (manifest, expected) in cases.iter().zip(["ID001", "ID004", "ID005", "ID002"]) {
        let diagnostics = bind_identity(&projection, manifest).unwrap_err();
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == expected),
            "expected {expected}, got {diagnostics:?}"
        );
    }
}

#[test]
fn identity_manifest_rejects_rename_without_explicit_evolution_map() {
    let projection = parse_oak("class A { renamed: uuid }")
        .unwrap()
        .project_schema()
        .unwrap();
    let manifest = IdentityManifest {
        format_version: IDENTITY_MANIFEST_VERSION.to_owned(),
        types: vec![TypeIdentity {
            canonical_path: vec!["A".to_owned()],
            type_id: 1,
            kind: TypeContractKind::Class,
            fields: vec![FieldIdentity {
                canonical_name: "old_name".to_owned(),
                field_id: 2,
                virtual_field_index: 0,
            }],
        }],
    };
    let diagnostics = bind_identity(&projection, &manifest).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "ID006")
    );
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "ID007")
    );
}

#[test]
fn identity_fingerprint_ignores_source_manifest_and_slot_order() {
    let first_projection = parse_oak("class B { b: utf8, a: uuid }\nclass A { id: uuid }")
        .unwrap()
        .project_schema()
        .unwrap();
    let second_projection = parse_oak("class A { id: uuid }\nclass B { a: uuid, b: utf8 }")
        .unwrap()
        .project_schema()
        .unwrap();
    let first_manifest = IdentityManifest {
        format_version: IDENTITY_MANIFEST_VERSION.to_owned(),
        types: vec![
            TypeIdentity {
                canonical_path: vec!["B".to_owned()],
                type_id: 3,
                kind: TypeContractKind::Class,
                fields: vec![
                    FieldIdentity {
                        canonical_name: "b".to_owned(),
                        field_id: 11,
                        virtual_field_index: 0,
                    },
                    FieldIdentity {
                        canonical_name: "a".to_owned(),
                        field_id: 12,
                        virtual_field_index: 1,
                    },
                ],
            },
            TypeIdentity {
                canonical_path: vec!["A".to_owned()],
                type_id: 7,
                kind: TypeContractKind::Class,
                fields: vec![FieldIdentity {
                    canonical_name: "id".to_owned(),
                    field_id: 10,
                    virtual_field_index: 0,
                }],
            },
        ],
    };
    let second_manifest = IdentityManifest {
        format_version: IDENTITY_MANIFEST_VERSION.to_owned(),
        types: vec![
            TypeIdentity {
                canonical_path: vec!["A".to_owned()],
                type_id: 7,
                kind: TypeContractKind::Class,
                fields: vec![FieldIdentity {
                    canonical_name: "id".to_owned(),
                    field_id: 10,
                    virtual_field_index: 0,
                }],
            },
            TypeIdentity {
                canonical_path: vec!["B".to_owned()],
                type_id: 3,
                kind: TypeContractKind::Class,
                fields: vec![
                    FieldIdentity {
                        canonical_name: "a".to_owned(),
                        field_id: 12,
                        virtual_field_index: 9,
                    },
                    FieldIdentity {
                        canonical_name: "b".to_owned(),
                        field_id: 11,
                        virtual_field_index: 8,
                    },
                ],
            },
        ],
    };
    let first = schema_fingerprint(&bind_identity(&first_projection, &first_manifest).unwrap());
    let second = schema_fingerprint(&bind_identity(&second_projection, &second_manifest).unwrap());
    assert_eq!(first, second);
    assert_eq!(first.len(), 64);
}

#[test]
fn identity_fingerprint_changes_for_semantic_changes() {
    let projection = parse_oak("class A { id: uuid }")
        .unwrap()
        .project_schema()
        .unwrap();
    let manifest = IdentityManifest {
        format_version: IDENTITY_MANIFEST_VERSION.to_owned(),
        types: vec![TypeIdentity {
            canonical_path: vec!["A".to_owned()],
            type_id: 1,
            kind: TypeContractKind::Class,
            fields: vec![FieldIdentity {
                canonical_name: "id".to_owned(),
                field_id: 2,
                virtual_field_index: 0,
            }],
        }],
    };
    let baseline = schema_fingerprint(&bind_identity(&projection, &manifest).unwrap());
    let changed_projection = parse_oak("class A { id: utf8 }")
        .unwrap()
        .project_schema()
        .unwrap();
    let changed = schema_fingerprint(&bind_identity(&changed_projection, &manifest).unwrap());
    assert_ne!(baseline, changed);
}

#[test]
fn identity_manifest_fixture_binds_and_produces_fingerprint() {
    let source = include_str!("../../../../specifications/fixtures/contracts/projection_basic.vos");
    let manifest_json = include_str!(
        "../../../../specifications/fixtures/contracts/identity_basic.manifest.json"
    );
    let projection = parse_oak(source).unwrap().project_schema().unwrap();
    let manifest = IdentityManifest::from_json(manifest_json).unwrap();
    let bound = bind_identity(&projection, &manifest).unwrap();
    let fingerprint = schema_fingerprint(&bound);
    assert_eq!(
        fingerprint,
        "14492965ddbf52183c30f003be100bab77160559be8d47cbef55b76e08a19b5f"
    );
}

#[test]
fn identity_manifest_reader_rejects_unknown_fields() {
    let mut value: Value = serde_json::from_str(include_str!(
        "../../../../specifications/fixtures/contracts/identity_basic.manifest.json"
    ))
    .unwrap();
    value["unexpected"] = Value::Bool(true);
    let error = IdentityManifest::from_json(&serde_json::to_string(&value).unwrap()).unwrap_err();
    assert_eq!(error.code, "ID011");
}
