use vos_types::{
    CanonicalScalar, EntityKind, ResolvedCanonicalType, ResolvedContract, ResolvedFieldContract,
    ResolvedTypeContract, SourceProvenance, StableId, StableIdentityEntry, StableIdentityManifest,
    TypeContractKind, STABLE_IDENTITY_MANIFEST_VERSION,
};

fn minimal_contract() -> ResolvedContract {
    ResolvedContract {
        format_version: "vos-resolved-contract-v1".to_owned(),
        identity_manifest_version: "vos-identity-manifest-v0".to_owned(),
        schema_fingerprint: "d73fdb40c7503a045d19e9f6476cb3b835700f455b229a46c70b7de1b04901f9"
            .to_owned(),
        types: vec![ResolvedTypeContract {
            type_id: 1,
            canonical_path: vec!["T".to_owned()],
            kind: TypeContractKind::Class,
            fields: vec![ResolvedFieldContract {
                field_id: 2,
                virtual_field_index: Some(0),
                canonical_name: "id".to_owned(),
                canonical_type: ResolvedCanonicalType::Scalar(CanonicalScalar::Uuid),
                attributes: vec![],
                default_value: None,
            }],
        }],
    }
}

#[test]
fn stable_identity_manifest_builder_from_resolved_contract_produces_entries() {
    let contract = minimal_contract();
    let manifest = StableIdentityManifest::from_resolved_contract(&contract);

    assert_eq!(manifest.format_version, STABLE_IDENTITY_MANIFEST_VERSION);
    assert_eq!(manifest.entries.len(), 2); // 1 type + 1 field

    let type_entry = manifest
        .entries
        .iter()
        .find(|entry| entry.stable_id == StableId::new(1))
        .expect("type entry must exist");
    assert_eq!(type_entry.entity_kind, EntityKind::Class);
    assert_eq!(type_entry.current_name, "T");
    assert_eq!(type_entry.provenance.source_unit_id, 0);
    assert_eq!(type_entry.previous, None);
    assert_eq!(type_entry.fingerprint.len(), 64);
    assert!(type_entry.fingerprint.bytes().all(|byte| byte.is_ascii_hexdigit()));

    let field_entry = manifest
        .entries
        .iter()
        .find(|entry| entry.stable_id == StableId::new(2))
        .expect("field entry must exist");
    assert_eq!(field_entry.entity_kind, EntityKind::Field);
    assert_eq!(field_entry.current_name, "id");
    assert_eq!(field_entry.previous, None);
    assert_eq!(field_entry.fingerprint.len(), 64);
}

#[test]
fn stable_identity_manifest_serialize_roundtrip_preserves_all_fields() {
    let contract = minimal_contract();
    let manifest = StableIdentityManifest::from_resolved_contract(&contract);

    let json = manifest
        .to_json()
        .expect("stable identity manifest serializes");
    let parsed = StableIdentityManifest::from_json(&json)
        .expect("stable identity manifest round-trips");

    assert_eq!(parsed, manifest);
    assert_eq!(parsed.entries.len(), manifest.entries.len());
}

#[test]
fn stable_identity_manifest_rejects_invalid_entries_on_roundtrip() {
    let bad = StableIdentityManifest {
        format_version: STABLE_IDENTITY_MANIFEST_VERSION.to_owned(),
        artifact_id: "test".to_owned(),
        entries: vec![StableIdentityEntry {
            stable_id: StableId::new(0),
            entity_kind: EntityKind::Class,
            current_name: "T".to_owned(),
            fingerprint: "a".repeat(64),
            provenance: SourceProvenance::unit(0),
            previous: None,
        }],
    };
    assert!(bad.validate().is_err());
    let json = bad.to_json().unwrap();
    assert!(StableIdentityManifest::from_json(&json).is_err());
}

#[test]
fn stable_identity_manifest_rejects_empty_name() {
    let bad = StableIdentityManifest {
        format_version: STABLE_IDENTITY_MANIFEST_VERSION.to_owned(),
        artifact_id: "test".to_owned(),
        entries: vec![StableIdentityEntry {
            stable_id: StableId::new(1),
            entity_kind: EntityKind::Class,
            current_name: "".to_owned(),
            fingerprint: "a".repeat(64),
            provenance: SourceProvenance::unit(0),
            previous: None,
        }],
    };
    assert!(bad.validate().is_err());
}

#[test]
fn stable_identity_manifest_rejects_short_fingerprint() {
    let bad = StableIdentityManifest {
        format_version: STABLE_IDENTITY_MANIFEST_VERSION.to_owned(),
        artifact_id: "test".to_owned(),
        entries: vec![StableIdentityEntry {
            stable_id: StableId::new(1),
            entity_kind: EntityKind::Class,
            current_name: "T".to_owned(),
            fingerprint: "short".to_owned(),
            provenance: SourceProvenance::unit(0),
            previous: None,
        }],
    };
    assert!(bad.validate().is_err());
}

#[test]
fn verify_reorder_stable_passes_for_same_entries() {
    let contract = minimal_contract();
    let before = StableIdentityManifest::from_resolved_contract(&contract);
    // Same entries, different order is impossible here because we rebuild from
    // the same ordered ResolvedContract — the test still confirms Ok(()) when
    // comparing a manifest against itself.
    let result = before.verify_reorder_stable(&before);
    assert!(result.is_ok(), "self-compare must pass: {result:?}");
}

#[test]
fn verify_reorder_stable_rejects_missing_entry() {
    let before = StableIdentityManifest {
        format_version: STABLE_IDENTITY_MANIFEST_VERSION.to_owned(),
        artifact_id: "test".to_owned(),
        entries: vec![
            StableIdentityEntry {
                stable_id: StableId::new(1),
                entity_kind: EntityKind::Class,
                current_name: "T".to_owned(),
                fingerprint: "a".repeat(64),
                provenance: SourceProvenance::unit(0),
                previous: None,
            },
            StableIdentityEntry {
                stable_id: StableId::new(2),
                entity_kind: EntityKind::Field,
                current_name: "id".to_owned(),
                fingerprint: "b".repeat(64),
                provenance: SourceProvenance::unit(0),
                previous: None,
            },
        ],
    };
    let current = StableIdentityManifest {
        entries: before.entries[..1].to_vec(),
        ..before.clone()
    };
    let errors = current
        .verify_reorder_stable(&before)
        .expect_err("removing an entry must fail reorder check");
    assert!(!errors.is_empty());
    assert!(errors.iter().any(|msg| msg.contains("reorder removed")));
}

#[test]
fn verify_rename_tracking_passes_for_stable_snapshot() {
    let contract = minimal_contract();
    let before = StableIdentityManifest::from_resolved_contract(&contract);
    let result = before.verify_rename_tracking(&before);
    assert!(result.is_ok(), "self-compare must pass: {result:?}");
}

#[test]
fn verify_rename_tracking_allows_name_change_with_same_stable_id() {
    let before = StableIdentityManifest {
        format_version: STABLE_IDENTITY_MANIFEST_VERSION.to_owned(),
        artifact_id: "test".to_owned(),
        entries: vec![StableIdentityEntry {
            stable_id: StableId::new(1),
            entity_kind: EntityKind::Class,
            current_name: "OldName".to_owned(),
            fingerprint: "a".repeat(64),
            provenance: SourceProvenance::unit(0),
            previous: None,
        }],
    };
    let current = StableIdentityManifest {
        entries: vec![StableIdentityEntry {
            current_name: "NewName".to_owned(),
            fingerprint: "b".repeat(64),
            ..before.entries[0].clone()
        }],
        ..before.clone()
    };
    // Same stable_id, name changed → legal rename, Ok(()).
    assert!(
        current.verify_rename_tracking(&before).is_ok(),
        "rename with same stable id must be allowed"
    );
}

#[test]
fn verify_rename_tracking_rejects_kind_change_on_same_stable_id() {
    let before = StableIdentityManifest {
        format_version: STABLE_IDENTITY_MANIFEST_VERSION.to_owned(),
        artifact_id: "test".to_owned(),
        entries: vec![StableIdentityEntry {
            stable_id: StableId::new(1),
            entity_kind: EntityKind::Class,
            current_name: "T".to_owned(),
            fingerprint: "a".repeat(64),
            provenance: SourceProvenance::unit(0),
            previous: None,
        }],
    };
    let current = StableIdentityManifest {
        entries: vec![StableIdentityEntry {
            entity_kind: EntityKind::Service,
            ..before.entries[0].clone()
        }],
        ..before.clone()
    };
    let errors = current
        .verify_rename_tracking(&before)
        .expect_err("changing entity kind on same stable id must fail");
    assert!(errors.iter().any(|msg| msg.contains("entity kind")));
}

#[test]
fn verify_rename_tracking_rejects_fingerprint_change_without_rename() {
    let before = StableIdentityManifest {
        format_version: STABLE_IDENTITY_MANIFEST_VERSION.to_owned(),
        artifact_id: "test".to_owned(),
        entries: vec![StableIdentityEntry {
            stable_id: StableId::new(1),
            entity_kind: EntityKind::Class,
            current_name: "T".to_owned(),
            fingerprint: "a".repeat(64),
            provenance: SourceProvenance::unit(0),
            previous: None,
        }],
    };
    let current = StableIdentityManifest {
        entries: vec![StableIdentityEntry {
            fingerprint: "b".repeat(64),
            ..before.entries[0].clone()
        }],
        ..before.clone()
    };
    let errors = current
        .verify_rename_tracking(&before)
        .expect_err("fingerprint change without rename must fail rename tracking");
    assert!(errors.iter().any(|msg| msg.contains("fingerprint without a rename")));
}

#[test]
fn entity_kind_classification_helpers() {
    assert!(EntityKind::Table.is_type());
    assert!(EntityKind::Class.is_type());
    assert!(!EntityKind::Field.is_type());
    assert!(!EntityKind::Operation.is_type());
    assert!(EntityKind::Field.is_field());
    assert!(EntityKind::Operation.is_operation());
    assert!(!EntityKind::Class.is_operation());
    assert!(EntityKind::Service.is_service());
    assert!(EntityKind::Method.is_service());
    assert!(!EntityKind::Field.is_service());
}

#[test]
fn stable_id_reserved_zero_is_invalid() {
    assert!(!StableId::RESERVED_ZERO.is_valid());
    assert!(!StableId::new(0).is_valid());
    assert!(StableId::new(42).is_valid());
}
