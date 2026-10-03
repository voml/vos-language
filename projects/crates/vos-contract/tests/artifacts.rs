use serde_json::{Value, json};
use vos_contract::{
    compare_identity, evolve_identity, bind_identity, schema_fingerprint, ContractDiagnostic,
    FieldIdentity, IdentityChange, IdentityHistory, IdentityManifest, SchemaProjection,
    ResolvedCanonicalType, TypeContractKind, TypeIdentity, resolve_identity_types,
    resolve_identity_units,
    IDENTITY_MANIFEST_VERSION, parse_oak,
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
fn identity_fingerprint_ignores_provenance_spans_and_attribute_syntax_spacing() {
    let first_projection = parse_oak("class A { [primary] id: uuid }")
        .unwrap()
        .project_schema()
        .unwrap();
    let second_projection = parse_oak("class A {\n  [primary]\n  id: uuid\n}")
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
    let first = schema_fingerprint(&bind_identity(&first_projection, &manifest).unwrap());
    let second = schema_fingerprint(&bind_identity(&second_projection, &manifest).unwrap());
    assert_eq!(first, second);
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

#[test]
fn identity_evolution_reports_explicit_changes_and_tombstones() {
    let previous_projection = parse_oak("class A { id: uuid, old: utf8 }")
        .unwrap()
        .project_schema()
        .unwrap();
    let previous_manifest = IdentityManifest {
        format_version: IDENTITY_MANIFEST_VERSION.to_owned(),
        types: vec![TypeIdentity {
            canonical_path: vec!["A".to_owned()],
            type_id: 1,
            kind: TypeContractKind::Class,
            fields: vec![
                FieldIdentity {
                    canonical_name: "id".to_owned(),
                    field_id: 2,
                    virtual_field_index: 0,
                },
                FieldIdentity {
                    canonical_name: "old".to_owned(),
                    field_id: 3,
                    virtual_field_index: 1,
                },
            ],
        }],
    };
    let current_projection = parse_oak("class Renamed { new_id: uuid, extra: bool }")
        .unwrap()
        .project_schema()
        .unwrap();
    let current_manifest = IdentityManifest {
        format_version: IDENTITY_MANIFEST_VERSION.to_owned(),
        types: vec![TypeIdentity {
            canonical_path: vec!["Renamed".to_owned()],
            type_id: 1,
            kind: TypeContractKind::Class,
            fields: vec![
                FieldIdentity {
                    canonical_name: "new_id".to_owned(),
                    field_id: 2,
                    virtual_field_index: 1,
                },
                FieldIdentity {
                    canonical_name: "extra".to_owned(),
                    field_id: 4,
                    virtual_field_index: 2,
                },
            ],
        }],
    };
    let previous = bind_identity(&previous_projection, &previous_manifest).unwrap();
    let current = bind_identity(&current_projection, &current_manifest).unwrap();
    let evolution = compare_identity(&previous, &current).unwrap();
    assert!(evolution.changes.contains(&IdentityChange::TypeRenamed {
        type_id: 1,
        from: vec!["A".to_owned()],
        to: vec!["Renamed".to_owned()],
    }));
    assert!(evolution.changes.contains(&IdentityChange::FieldRenamed {
        type_id: 1,
        field_id: 2,
        from: "id".to_owned(),
        to: "new_id".to_owned(),
    }));
    assert!(evolution.changes.contains(&IdentityChange::FieldReordered {
        type_id: 1,
        field_id: 2,
        from: 0,
        to: 1,
    }));
    assert!(evolution.changes.contains(&IdentityChange::FieldRemoved {
        type_id: 1,
        field_id: 3,
        canonical_name: "old".to_owned(),
        virtual_field_index: 1,
    }));
    assert!(evolution.changes.contains(&IdentityChange::FieldAdded {
        type_id: 1,
        field_id: 4,
        canonical_name: "extra".to_owned(),
        virtual_field_index: 2,
    }));
}

#[test]
fn identity_evolution_rejects_implicit_id_rebinding() {
    let previous = bind_identity(
        &parse_oak("class A { id: uuid }").unwrap().project_schema().unwrap(),
        &IdentityManifest {
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
        },
    )
    .unwrap();
    let current_type_rebound = bind_identity(
        &parse_oak("class A { id: uuid }").unwrap().project_schema().unwrap(),
        &IdentityManifest {
            format_version: IDENTITY_MANIFEST_VERSION.to_owned(),
            types: vec![TypeIdentity {
                canonical_path: vec!["A".to_owned()],
                type_id: 9,
                kind: TypeContractKind::Class,
                fields: vec![FieldIdentity {
                    canonical_name: "id".to_owned(),
                    field_id: 8,
                    virtual_field_index: 0,
                }],
            }],
        },
    )
    .unwrap();
    let diagnostics = compare_identity(&previous, &current_type_rebound).unwrap_err();
    assert!(diagnostics.iter().any(|item| item.code == "ID012"));

    let current_field_rebound = bind_identity(
        &parse_oak("class A { id: uuid }").unwrap().project_schema().unwrap(),
        &IdentityManifest {
            format_version: IDENTITY_MANIFEST_VERSION.to_owned(),
            types: vec![TypeIdentity {
                canonical_path: vec!["A".to_owned()],
                type_id: 1,
                kind: TypeContractKind::Class,
                fields: vec![FieldIdentity {
                    canonical_name: "id".to_owned(),
                    field_id: 8,
                    virtual_field_index: 0,
                }],
            }],
        },
    )
    .unwrap();
    let diagnostics = compare_identity(&previous, &current_field_rebound).unwrap_err();
    assert!(diagnostics.iter().any(|item| item.code == "ID013"));
}

#[test]
fn identity_history_records_tombstones_and_rejects_reuse() {
    let previous = bind_identity(
        &parse_oak("class A { id: uuid, old: utf8 }")
            .unwrap()
            .project_schema()
            .unwrap(),
        &IdentityManifest {
            format_version: IDENTITY_MANIFEST_VERSION.to_owned(),
            types: vec![TypeIdentity {
                canonical_path: vec!["A".to_owned()],
                type_id: 1,
                kind: TypeContractKind::Class,
                fields: vec![
                    FieldIdentity {
                        canonical_name: "id".to_owned(),
                        field_id: 2,
                        virtual_field_index: 0,
                    },
                    FieldIdentity {
                        canonical_name: "old".to_owned(),
                        field_id: 3,
                        virtual_field_index: 1,
                    },
                ],
            }],
        },
    )
    .unwrap();
    let initial = IdentityHistory {
        format_version: vos_contract::IDENTITY_HISTORY_FORMAT_VERSION.to_owned(),
        manifest_version: IDENTITY_MANIFEST_VERSION.to_owned(),
        revision: 0,
        layout_epoch: 0,
        snapshot: previous,
        retired_types: Vec::new(),
        retired_fields: Vec::new(),
    };
    let current = bind_identity(
        &parse_oak("class A { id: uuid }")
            .unwrap()
            .project_schema()
            .unwrap(),
        &IdentityManifest {
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
        },
    )
    .unwrap();
    let evolved = evolve_identity(&initial, current.clone()).unwrap();
    assert_eq!(evolved.revision, 1);
    assert_eq!(evolved.layout_epoch, 1);
    assert_eq!(evolved.retired_fields.len(), 1);
    assert_eq!(evolved.retired_fields[0].field_id, 3);
    let serialized = evolved.to_json().unwrap();
    assert_eq!(IdentityHistory::from_json(&serialized).unwrap(), evolved);
    let mut corrupted: Value = serde_json::from_str(&serialized).unwrap();
    corrupted["retiredFields"][0]["fieldId"] = json!(2);
    assert_eq!(IdentityHistory::from_json(&corrupted.to_string()).unwrap_err().code, "ID023");
    let mut unknown: Value = serde_json::from_str(&serialized).unwrap();
    unknown["unexpected"] = json!(true);
    assert_eq!(IdentityHistory::from_json(&unknown.to_string()).unwrap_err().code, "ID020");
    for (pointer, value, code) in [
        ("/snapshot/manifestVersion", json!("unsupported"), "ID021"),
        ("/snapshot/types/0/typeId", json!(0), "ID024"),
        ("/snapshot/types/0/canonicalPath", json!([""]), "ID024"),
        ("/snapshot/types/0/fields/0/fieldId", json!(0), "ID025"),
        ("/snapshot/types/0/fields/0/canonicalName", json!(""), "ID025"),
        ("/retiredFields/0/retiredAtRevision", json!(2), "ID023"),
        ("/retiredFields/0/typeId", json!(99), "ID023"),
    ] {
        let mut invalid: Value = serde_json::from_str(&serialized).unwrap();
        *invalid.pointer_mut(pointer).unwrap() = value;
        assert_eq!(IdentityHistory::from_json(&invalid.to_string()).unwrap_err().code, code, "{pointer}");
    }
    for (pointer, code) in [("/snapshot/types", "ID024"), ("/snapshot/types/0/fields", "ID025")] {
        let mut invalid: Value = serde_json::from_str(&serialized).unwrap();
        let entries = invalid.pointer_mut(pointer).unwrap().as_array_mut().unwrap();
        entries.push(entries[0].clone());
        assert_eq!(IdentityHistory::from_json(&invalid.to_string()).unwrap_err().code, code, "{pointer}");
    }

    let reused = bind_identity(
        &parse_oak("class A { id: uuid, replacement: bool }")
            .unwrap()
            .project_schema()
            .unwrap(),
        &IdentityManifest {
            format_version: IDENTITY_MANIFEST_VERSION.to_owned(),
            types: vec![TypeIdentity {
                canonical_path: vec!["A".to_owned()],
                type_id: 1,
                kind: TypeContractKind::Class,
                fields: vec![
                    FieldIdentity {
                        canonical_name: "id".to_owned(),
                        field_id: 2,
                        virtual_field_index: 0,
                    },
                    FieldIdentity {
                        canonical_name: "replacement".to_owned(),
                        field_id: 3,
                        virtual_field_index: 1,
                    },
                ],
            }],
        },
    )
    .unwrap();
    let diagnostics = evolve_identity(&evolved, reused).unwrap_err();
    assert!(diagnostics.iter().any(|item| item.code == "ID016"));
}

#[test]
fn identity_type_resolution_binds_user_types_and_keeps_builtins() {
    let projection = parse_oak("class Address { city: utf8 }\nclass User { address: Address?, id: uuid }")
        .unwrap()
        .project_schema()
        .unwrap();
    let manifest = IdentityManifest {
        format_version: IDENTITY_MANIFEST_VERSION.to_owned(),
        types: vec![
            TypeIdentity {
                canonical_path: vec!["Address".to_owned()],
                type_id: 7,
                kind: TypeContractKind::Class,
                fields: vec![FieldIdentity {
                    canonical_name: "city".to_owned(),
                    field_id: 8,
                    virtual_field_index: 0,
                }],
            },
            TypeIdentity {
                canonical_path: vec!["User".to_owned()],
                type_id: 9,
                kind: TypeContractKind::Class,
                fields: vec![
                    FieldIdentity {
                        canonical_name: "address".to_owned(),
                        field_id: 10,
                        virtual_field_index: 0,
                    },
                    FieldIdentity {
                        canonical_name: "id".to_owned(),
                        field_id: 11,
                        virtual_field_index: 1,
                    },
                ],
            },
        ],
    };
    let bound = bind_identity(&projection, &manifest).unwrap();
    let resolved = resolve_identity_types(&bound).unwrap();
    assert!(matches!(
        &resolved.types[1].fields[0].canonical_type,
        ResolvedCanonicalType::Optional(inner)
            if matches!(inner.as_ref(), ResolvedCanonicalType::User { type_id: 7, .. })
    ));
    assert_eq!(
        resolved.types[1].fields[1].canonical_type,
        ResolvedCanonicalType::Builtin(vec!["uuid".to_owned()])
    );
}

#[test]
fn identity_type_resolution_rejects_unknown_user_types() {
    let projection = parse_oak("class User { address: Missing }")
        .unwrap()
        .project_schema()
        .unwrap();
    let manifest = IdentityManifest {
        format_version: IDENTITY_MANIFEST_VERSION.to_owned(),
        types: vec![TypeIdentity {
            canonical_path: vec!["User".to_owned()],
            type_id: 1,
            kind: TypeContractKind::Class,
            fields: vec![FieldIdentity {
                canonical_name: "address".to_owned(),
                field_id: 2,
                virtual_field_index: 0,
            }],
        }],
    };
    let bound = bind_identity(&projection, &manifest).unwrap();
    let diagnostics = resolve_identity_types(&bound).unwrap_err();
    assert!(diagnostics.iter().any(|item| item.code == "RES001"));
}

#[test]
fn relative_and_qualified_types_resolve_to_the_same_namespace_identity() {
    let source = "namespace demo\nclass Address { city: utf8 }\nclass User { local: Address?, qualified: demo::Address }";
    let projection = parse_oak(source).unwrap().project_schema().unwrap();
    let manifest = IdentityManifest {
        format_version: IDENTITY_MANIFEST_VERSION.to_owned(),
        types: vec![
            TypeIdentity {
                canonical_path: vec!["demo".to_owned(), "Address".to_owned()],
                type_id: 1,
                kind: TypeContractKind::Class,
                fields: vec![FieldIdentity { canonical_name: "city".to_owned(), field_id: 1, virtual_field_index: 0 }],
            },
            TypeIdentity {
                canonical_path: vec!["demo".to_owned(), "User".to_owned()],
                type_id: 2,
                kind: TypeContractKind::Class,
                fields: vec![
                    FieldIdentity { canonical_name: "local".to_owned(), field_id: 2, virtual_field_index: 0 },
                    FieldIdentity { canonical_name: "qualified".to_owned(), field_id: 3, virtual_field_index: 1 },
                ],
            },
        ],
    };
    let bound = bind_identity(&projection, &manifest).unwrap();
    let resolved = resolve_identity_types(&bound).unwrap();
    let expected = ResolvedCanonicalType::User { path: vec!["demo".to_owned(), "Address".to_owned()], type_id: 1 };
    assert_eq!(resolved.types[1].fields[0].canonical_type, ResolvedCanonicalType::Optional(Box::new(expected.clone())));
    assert_eq!(resolved.types[1].fields[1].canonical_type, expected);
}

#[test]
fn multi_unit_resolution_joins_global_symbol_table_without_reparsing() {
    let users = parse_oak("namespace demo\nclass User { address: Address }")
        .unwrap()
        .project_schema()
        .unwrap();
    let addresses = parse_oak("namespace demo\nclass Address { city: utf8 }")
        .unwrap()
        .project_schema()
        .unwrap();
    let user_manifest = IdentityManifest {
        format_version: IDENTITY_MANIFEST_VERSION.to_owned(),
        types: vec![TypeIdentity {
            canonical_path: vec!["demo".to_owned(), "User".to_owned()],
            type_id: 10,
            kind: TypeContractKind::Class,
            fields: vec![FieldIdentity {
                canonical_name: "address".to_owned(),
                field_id: 11,
                virtual_field_index: 0,
            }],
        }],
    };
    let address_manifest = IdentityManifest {
        format_version: IDENTITY_MANIFEST_VERSION.to_owned(),
        types: vec![TypeIdentity {
            canonical_path: vec!["demo".to_owned(), "Address".to_owned()],
            type_id: 20,
            kind: TypeContractKind::Class,
            fields: vec![FieldIdentity {
                canonical_name: "city".to_owned(),
                field_id: 21,
                virtual_field_index: 0,
            }],
        }],
    };
    let users = bind_identity(&users, &user_manifest).unwrap();
    let addresses = bind_identity(&addresses, &address_manifest).unwrap();
    let resolved = resolve_identity_units(&[&users, &addresses]).unwrap();
    assert_eq!(resolved.types.len(), 2);
    assert_eq!(resolved.types[0].canonical_path, ["demo", "User"]);
    assert_eq!(
        resolved.types[0].fields[0].canonical_type,
        ResolvedCanonicalType::User {
            path: vec!["demo".to_owned(), "Address".to_owned()],
            type_id: 20,
        }
    );
}

#[test]
fn multi_unit_resolution_rejects_duplicate_paths_ids_and_versions() {
    let projection = parse_oak("class A { id: uuid }").unwrap().project_schema().unwrap();
    let first = bind_identity(
        &projection,
        &IdentityManifest {
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
        },
    )
    .unwrap();
    let duplicate_path = first.clone();
    let mut different_version = first.clone();
    different_version.manifest_version = "other-manifest".to_owned();
    let duplicate_ids = bind_identity(
        &parse_oak("class B { id: uuid }").unwrap().project_schema().unwrap(),
        &IdentityManifest {
            format_version: IDENTITY_MANIFEST_VERSION.to_owned(),
            types: vec![TypeIdentity {
                canonical_path: vec!["B".to_owned()],
                type_id: 1,
                kind: TypeContractKind::Class,
                fields: vec![FieldIdentity {
                    canonical_name: "id".to_owned(),
                    field_id: 2,
                    virtual_field_index: 0,
                }],
            }],
        },
    )
    .unwrap();
    assert!(resolve_identity_units(&[&first, &duplicate_path])
        .unwrap_err()
        .iter()
        .any(|item| item.code == "RES004"));
    assert!(resolve_identity_units(&[&first, &different_version])
        .unwrap_err()
        .iter()
        .any(|item| item.code == "RES003"));
    let diagnostics = resolve_identity_units(&[&first, &duplicate_ids]).unwrap_err();
    assert!(diagnostics.iter().any(|item| item.code == "RES007"));
    assert!(diagnostics.iter().any(|item| item.code == "RES006"));
}
