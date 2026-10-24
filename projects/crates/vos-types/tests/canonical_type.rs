use vos_types::{
    CanonicalScalar, ResolvedCanonicalType, ResolvedCanonicalTypeArgument, ResolvedFieldContract,
    ResolvedTypeContract, TypeContractKind, resolved_schema_fingerprint,
};

fn scalar_type(scalar: CanonicalScalar) -> ResolvedCanonicalType {
    ResolvedCanonicalType::Scalar(scalar)
}

fn contract_with(canonical_type: ResolvedCanonicalType) -> Vec<ResolvedTypeContract> {
    vec![ResolvedTypeContract {
        type_id: 1,
        canonical_path: vec!["T".to_owned()],
        kind: TypeContractKind::Class,
        fields: vec![ResolvedFieldContract {
            field_id: 2,
            virtual_field_index: Some(0),
            canonical_name: "value".to_owned(),
            canonical_type,
            attributes: Vec::new(),
            default_value: None,
        }],
    }]
}

#[test]
fn every_canonical_scalar_round_trips_through_its_name() {
    assert_eq!(CanonicalScalar::ALL.len(), 19);
    for scalar in CanonicalScalar::ALL {
        assert_eq!(CanonicalScalar::from_name(scalar.as_str()), Some(scalar));
        assert_eq!(scalar.to_string(), scalar.as_str());
    }
    assert_eq!(CanonicalScalar::ALL[0], CanonicalScalar::I8);
    assert_eq!(CanonicalScalar::ALL[18], CanonicalScalar::Bytes);
}

#[test]
fn source_aliases_normalize_to_canonical_scalars() {
    assert_eq!(
        CanonicalScalar::from_source_name("d128"),
        Some(CanonicalScalar::Decimal)
    );
    assert_eq!(
        CanonicalScalar::from_source_name("decimal"),
        Some(CanonicalScalar::Decimal)
    );
    assert_eq!(CanonicalScalar::from_source_name("string"), None);
    assert_eq!(CanonicalScalar::from_source_name("number"), None);
    assert_eq!(CanonicalScalar::from_source_name("json"), None);
    assert_eq!(CanonicalScalar::from_source_name("Utf8"), None);
}

#[test]
fn scalar_wire_form_is_a_tagged_name_not_a_path() {
    let wire = serde_json::to_string(&scalar_type(CanonicalScalar::DateTime)).unwrap();
    assert_eq!(wire, r#"{"scalar":"datetime"}"#);

    let decoded: ResolvedCanonicalType = serde_json::from_str(r#"{"scalar":"decimal"}"#).unwrap();
    assert_eq!(decoded, scalar_type(CanonicalScalar::Decimal));

    assert!(
        serde_json::from_str::<ResolvedCanonicalType>(r#"{"scalar":"string"}"#).is_err(),
        "unknown scalars must not deserialize"
    );
    assert!(
        serde_json::from_str::<ResolvedCanonicalType>(r#"{"builtin":["uuid"]}"#).is_err(),
        "the removed path-shaped scalar form must not deserialize"
    );
}

#[test]
fn wrappers_and_user_references_compare_structurally() {
    let uuid = scalar_type(CanonicalScalar::Uuid);
    let text = scalar_type(CanonicalScalar::Utf8);

    assert_eq!(
        ResolvedCanonicalType::Optional(Box::new(uuid.clone())),
        ResolvedCanonicalType::Optional(Box::new(uuid.clone()))
    );
    assert_ne!(
        ResolvedCanonicalType::Optional(Box::new(uuid.clone())),
        ResolvedCanonicalType::Optional(Box::new(text.clone()))
    );
    assert_ne!(
        ResolvedCanonicalType::List(Box::new(uuid.clone())),
        ResolvedCanonicalType::Optional(Box::new(uuid.clone()))
    );
    assert_ne!(
        ResolvedCanonicalType::Reference(Box::new(uuid.clone())),
        ResolvedCanonicalType::List(Box::new(uuid.clone()))
    );
    assert_ne!(
        ResolvedCanonicalType::User {
            path: vec!["T".to_owned()],
            type_id: 1,
        },
        ResolvedCanonicalType::User {
            path: vec!["T".to_owned()],
            type_id: 2,
        }
    );
    assert_ne!(
        ResolvedCanonicalType::Generic {
            path: vec!["vector".to_owned()],
            arguments: vec![ResolvedCanonicalTypeArgument::Literal("3".to_owned())],
        },
        ResolvedCanonicalType::Generic {
            path: vec!["vector".to_owned()],
            arguments: vec![ResolvedCanonicalTypeArgument::Literal("4".to_owned())],
        }
    );
}

#[test]
fn canonical_scalar_participates_in_the_resolved_fingerprint() {
    let uuid = resolved_schema_fingerprint(&contract_with(scalar_type(CanonicalScalar::Uuid)));
    let uuid_again =
        resolved_schema_fingerprint(&contract_with(scalar_type(CanonicalScalar::Uuid)));
    let text = resolved_schema_fingerprint(&contract_with(scalar_type(CanonicalScalar::Utf8)));
    let optional_uuid = resolved_schema_fingerprint(&contract_with(
        ResolvedCanonicalType::Optional(Box::new(scalar_type(CanonicalScalar::Uuid))),
    ));

    assert_eq!(uuid, uuid_again);
    assert_eq!(uuid.len(), 64);
    assert_ne!(uuid, text);
    assert_ne!(uuid, optional_uuid);
}
