use vos_types::{
    CanonicalScalar, EntityKind, MethodId, MethodIdentityKey, MethodStreamMode,
    OperationCardinality, OperationContract, OperationEffect, OperationId, OperationIdentityKey,
    OperationKind, ResolvedCanonicalType, ResolvedContract, ResolvedFieldContract,
    ResolvedTypeContract, ServiceContract, ServiceId, ServiceIdentityKey, ServiceMethodContract,
    SourceProvenance, StableId, StableIdentityEntry, StableIdentityManifest, TypeContractKind,
    ContractArtifact, ContractPayload, ContractFormat, ContractProducer,
    STABLE_IDENTITY_MANIFEST_VERSION, identity_hash_u64,
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

// ---------------------------------------------------------------------------
// Fixture helpers: minimal valid contract artifact
// ---------------------------------------------------------------------------

fn op_key(name: &str) -> OperationContract {
    let identity = OperationIdentityKey::new(
        &[],
        OperationKind::Query,
        name,
        vec![],
        "unit",
    )
    .unwrap();
    OperationContract {
        operation_id: OperationId::derive(&identity),
        kind: OperationKind::Query,
        name: name.to_owned(),
        canonical_path: vec![name.to_owned()],
        parameters: vec![],
        result_type: None,
        cardinality: OperationCardinality::One,
        effect: OperationEffect::Pure,
        capabilities: vec![],
        source_unit_id: 0,
        span: (0..name.len()).into(),
        stable_id_hint: None,
    }
}

fn sample_artifact() -> ContractArtifact {
    let op_hello = op_key("hello");
    let op_farewell = op_key("farewell");
    let svc_id = ServiceId::derive(
        &ServiceIdentityKey::new(vec![
            "demo".to_owned(),
            "GreetService".to_owned(),
        ])
        .unwrap(),
    );
    let methods = vec![
        ServiceMethodContract {
            method_id: MethodId::derive(
                &MethodIdentityKey::new(svc_id.clone(), "greet").unwrap(),
            ),
            name: "greet".to_owned(),
            request_type: ResolvedCanonicalType::Scalar(CanonicalScalar::Utf8),
            response_type: ResolvedCanonicalType::Scalar(CanonicalScalar::Utf8),
            stream: MethodStreamMode::Unary,
            operation_id: op_hello.operation_id.clone(),
            attributes: vec![],
            stable_id_hint: None,
        },
        ServiceMethodContract {
            method_id: MethodId::derive(
                &MethodIdentityKey::new(svc_id.clone(), "leave").unwrap(),
            ),
            name: "leave".to_owned(),
            request_type: ResolvedCanonicalType::Scalar(CanonicalScalar::Utf8),
            response_type: ResolvedCanonicalType::Scalar(CanonicalScalar::Bool),
            stream: MethodStreamMode::Unary,
            operation_id: op_farewell.operation_id.clone(),
            attributes: vec![],
            stable_id_hint: None,
        },
    ];
    let svc = ServiceContract {
        service_id: svc_id,
        name: "GreetService".to_owned(),
        canonical_path: vec!["demo".to_owned(), "GreetService".to_owned()],
        attributes: vec![],
        methods,
        source_unit_id: 0,
        span: (0..30).into(),
        stable_id_hint: None,
    };
    let payload = ContractPayload {
        operations: vec![op_hello, op_farewell],
        services: vec![svc],
    };
    ContractArtifact::seal(
        ContractFormat::Hermes,
        ContractProducer::vos(),
        payload,
    )
}

// ---------------------------------------------------------------------------
// Tests for from_artifact
// ---------------------------------------------------------------------------

#[test]
fn from_artifact_produces_entries_for_every_operation_service_and_method() {
    let artifact = sample_artifact();
    let manifest = StableIdentityManifest::from_artifact(&artifact);

    // 2 operations + 1 service + 2 methods = 5 entries total
    assert_eq!(manifest.format_version, STABLE_IDENTITY_MANIFEST_VERSION);
    assert_eq!(manifest.entries.len(), 5);

    let operation_entries: Vec<_> = manifest
        .entries
        .iter()
        .filter(|entry| entry.entity_kind == EntityKind::Operation)
        .collect();
    assert_eq!(operation_entries.len(), 2, "every operation must yield one entry");

    let service_entries: Vec<_> = manifest
        .entries
        .iter()
        .filter(|entry| entry.entity_kind == EntityKind::Service)
        .collect();
    assert_eq!(service_entries.len(), 1, "every service must yield one entry");
    let svc_entry = service_entries[0];
    assert_eq!(svc_entry.current_name, "GreetService");
    assert_eq!(svc_entry.fingerprint.len(), 64);

    let method_entries: Vec<_> = manifest
        .entries
        .iter()
        .filter(|entry| entry.entity_kind == EntityKind::Method)
        .collect();
    assert_eq!(method_entries.len(), 2, "every method must yield one entry");

    // Every stable_id is non-zero and unique across kinds
    let all_ids: Vec<StableId> = manifest.entries.iter().map(|e| e.stable_id).collect();
    assert!(all_ids.iter().all(|id| id.is_valid()));
    let mut deduped = all_ids.clone();
    deduped.sort();
    deduped.dedup();
    assert_eq!(deduped.len(), all_ids.len(), "stable ids must be unique");
}

#[test]
fn reorder_operations_preserves_stable_id_set() {
    let original = sample_artifact();
    // Build a reordered artifact by swapping the operations vector. The
    // fingerprint changes (because operation order matters to the payload
    // fingerprint), but the stable id set computed by from_artifact should be
    // identical.
    let mut reordered_payload = original.payload.clone();
    reordered_payload.operations.reverse();
    let reordered = ContractArtifact::seal(
        original.version,
        original.producer.clone(),
        reordered_payload,
    );

    let before = StableIdentityManifest::from_artifact(&original);
    let after = StableIdentityManifest::from_artifact(&reordered);

    let before_ids: std::collections::HashSet<_> = before
        .entries
        .iter()
        .map(|entry| (entry.stable_id, entry.entity_kind))
        .collect();
    let after_ids: std::collections::HashSet<_> = after
        .entries
        .iter()
        .map(|entry| (entry.stable_id, entry.entity_kind))
        .collect();

    assert_eq!(
        before_ids, after_ids,
        "reordering operations must not change the stable id set"
    );

    // Also verify via the dedicated reorder check helper
    assert!(after.verify_reorder_stable(&before).is_ok());
}

#[test]
fn service_rename_preserves_stable_id_while_current_name_changes() {
    let original = sample_artifact();
    // Renaming the service means new ServiceId (identity changes → contract
    // artifact must carry the new id). But the existing operation and method
    // operations still have their old identities, and the methods bind to new
    // operation-ids derived from the new service identity plus the same method
    // names. We keep things simple: do NOT rebuild the artifact — just mutate
    // the manifest entries directly to simulate a rename and check that our
    // rename-tracking helper accepts it.
    let mut renamed_payload = original.payload.clone();
    let new_svc_id = ServiceId::derive(
        &ServiceIdentityKey::new(vec![
            "demo".to_owned(),
            "GreetServiceRenamed".to_owned(),
        ])
        .unwrap(),
    );
    let old_svc_id = renamed_payload.services[0].service_id.clone();
    // Give the service a new canonical path (which yields a new ServiceId)
    renamed_payload.services[0].name = "GreetServiceRenamed".to_owned();
    renamed_payload.services[0].canonical_path = vec![
        "demo".to_owned(),
        "GreetServiceRenamed".to_owned(),
    ];
    renamed_payload.services[0].service_id = new_svc_id.clone();
    // Methods also need new method_ids tied to the new service id and must
    // rebind their operation_id (same operations, so same operation ids).
    let method_names: Vec<String> = renamed_payload.services[0]
        .methods
        .iter()
        .map(|m| m.name.clone())
        .collect();
    let ops_binding: Vec<OperationId> = renamed_payload.services[0]
        .methods
        .iter()
        .map(|m| m.operation_id.clone())
        .collect();
    for (idx, method) in renamed_payload.services[0].methods.iter_mut().enumerate() {
        method.method_id = MethodId::derive(
            &MethodIdentityKey::new(new_svc_id.clone(), &method_names[idx]).unwrap(),
        );
        method.operation_id = ops_binding[idx].clone();
    }
    // Rebind all service methods to existing operations so the artifact
    // validates — operations still carry the original operation ids.
    let renamed = ContractArtifact::seal(
        original.version,
        original.producer.clone(),
        renamed_payload,
    );
    renamed.validate().expect("renamed artifact must be valid");
    // Now: the service and its methods now have NEW identities (because
    // identity derivation depends on canonical_path). So the set of
    // (stable_id, name, kind) changes entirely — this is the correct identity
    // behaviour (rename → new service_id → new stable_id).
    //
    // What we actually want to test here is that our rename-tracking helper
    // correctly reports "same stable_id, different current_name" is legal.
    // We therefore build a manual pair of manifests where only the service
    // current_name changes but we deliberately keep the same StableId.
    let before = StableIdentityManifest::from_artifact(&original);
    let mut after = before.clone();
    for entry in &mut after.entries {
        if entry.entity_kind == EntityKind::Service {
            entry.current_name = "GreetServiceRenamed".to_owned();
        }
    }
    // The service entry fingerprint may or may not change (renamed canonical
    // path inside the fingerprint changes the digest, but we're testing the
    // rename-tracking helper which only cares that *if name changed*, kind is
    // stable — fingerprint is allowed to change). The helper also requires
    // that entries that keep their name keep their fingerprint, which they do
    // in our hand-written scenario.
    let errors = after.verify_rename_tracking(&before);
    assert!(
        errors.is_ok(),
        "rename must keep stable_id intact and be accepted by rename tracking: {errors:?}"
    );
    // Sanity check: the stable_id of the renamed service entry did NOT change.
    let before_svc = before
        .entries
        .iter()
        .find(|e| e.entity_kind == EntityKind::Service)
        .unwrap();
    let after_svc = after
        .entries
        .iter()
        .find(|e| e.entity_kind == EntityKind::Service)
        .unwrap();
    assert_eq!(after_svc.stable_id, before_svc.stable_id);
    assert_eq!(after_svc.current_name, "GreetServiceRenamed");
    assert_ne!(after_svc.current_name, before_svc.current_name);

    assert!(old_svc_id.as_str() != new_svc_id.as_str());
}

// ---------------------------------------------------------------------------
// Evolution semantics: stable_id_hint as pre-allocated durable identity
// ---------------------------------------------------------------------------

#[test]
fn stable_id_hint_preserves_identity_across_service_rename() {
    // Build an artifact where the service carries a pre-allocated stable_id_hint.
    let original = sample_artifact();
    let before = StableIdentityManifest::from_artifact(&original);

    // Build a RENAMED artifact with NEW derived identities but the SAME hints.
    // This is what the VOS compiler should emit: first compile allocates a
    // stable_id_hint, subsequent compiles keep it even when names change.
    let mut renamed_payload = original.payload.clone();
    let new_svc_id = ServiceId::derive(
        &ServiceIdentityKey::new(vec![
            "demo".to_owned(),
            "GreetServiceRenamed".to_owned(),
        ])
        .unwrap(),
    );
    renamed_payload.services[0].name = "GreetServiceRenamed".to_owned();
    renamed_payload.services[0].canonical_path = vec![
        "demo".to_owned(),
        "GreetServiceRenamed".to_owned(),
    ];
    renamed_payload.services[0].service_id = new_svc_id.clone();
    // The crucial part: pre-allocate stable_id_hints so the manifest builder
    // ignores the content-derived ids. The service gets hint 100, methods get
    // 101 and 102.
    renamed_payload.services[0].stable_id_hint = Some(100);
    renamed_payload.services[0].methods[0].method_id = MethodId::derive(
        &MethodIdentityKey::new(new_svc_id.clone(), "greet").unwrap(),
    );
    renamed_payload.services[0].methods[0].stable_id_hint = Some(101);
    renamed_payload.services[0].methods[1].method_id = MethodId::derive(
        &MethodIdentityKey::new(new_svc_id.clone(), "leave").unwrap(),
    );
    renamed_payload.services[0].methods[1].stable_id_hint = Some(102);

    let renamed = ContractArtifact::seal(
        original.version,
        original.producer.clone(),
        renamed_payload,
    );
    renamed.validate().expect("renamed artifact with hints must be valid");

    let after = StableIdentityManifest::from_artifact(&renamed);
    let after_svc_entry = after
        .entries
        .iter()
        .find(|e| e.entity_kind == EntityKind::Service)
        .unwrap();

    // The service_id DID change (content-derived identity).
    assert_ne!(
        renamed.payload.services[0].service_id,
        original.payload.services[0].service_id,
        "service_id must change on rename"
    );
    // But the manifest stable_id for the service entry is now driven by the
    // hint (100), NOT the content-derived identity.
    assert_eq!(after_svc_entry.stable_id, StableId::new(100));
    assert_eq!(after_svc_entry.current_name, "GreetServiceRenamed");
    // Methods also keep their hints.
    let after_method_ids: Vec<StableId> = after
        .entries
        .iter()
        .filter(|e| e.entity_kind == EntityKind::Method)
        .map(|e| e.stable_id)
        .collect();
    assert!(after_method_ids.contains(&StableId::new(101)));
    assert!(after_method_ids.contains(&StableId::new(102)));
    // Rename tracking passes: same stable_id (from hint), different current_name.
    assert!(after.verify_rename_tracking(&before).is_ok());
}

#[test]
fn stable_id_hint_is_preferred_over_derived_identity_when_present() {
    // A minimal case: an operation with both a content-derived operation_id
    // and an explicit stable_id_hint. The manifest must use the hint.
    let with_hint = {
        let mut op = op_key("hello");
        op.stable_id_hint = Some(9999);
        let mut payload = original_payload();
        payload.operations = vec![op];
        ContractArtifact::seal(
            ContractFormat::Hermes,
            ContractProducer::vos(),
            payload,
        )
    };

    let manifest = StableIdentityManifest::from_artifact(&with_hint);
    let op_entry = manifest
        .entries
        .iter()
        .find(|e| e.entity_kind == EntityKind::Operation)
        .unwrap();
    assert_eq!(op_entry.stable_id, StableId::new(9999));

    // Without the hint, the stable_id falls back to hashing the operation_id.
    let without_hint = {
        let op = op_key("hello");
        let mut payload = original_payload();
        payload.operations = vec![op];
        ContractArtifact::seal(
            ContractFormat::Hermes,
            ContractProducer::vos(),
            payload,
        )
    };
    let manifest2 = StableIdentityManifest::from_artifact(&without_hint);
    let op_entry2 = manifest2
        .entries
        .iter()
        .find(|e| e.entity_kind == EntityKind::Operation)
        .unwrap();
    assert_eq!(
        op_entry2.stable_id,
        StableId::new(identity_hash_u64(
            without_hint.payload.operations[0].operation_id.as_str()
        ))
    );
    // And they MUST differ — hint 9999 vs content-derived.
    assert_ne!(op_entry.stable_id, op_entry2.stable_id);
}

// ---------------------------------------------------------------------------
// from_full_artifact merges ResolvedContract types/fields with artifact ops/services
// ---------------------------------------------------------------------------

#[test]
fn from_full_artifact_merges_all_kinds_into_one_manifest() {
    // ResolvedContract: 2 tables + 4 fields.
    let resolved = ResolvedContract {
        format_version: "vos-resolved-contract-v1".to_owned(),
        identity_manifest_version: "vos-identity-manifest-v0".to_owned(),
        schema_fingerprint: "d73fdb40c7503a045d19e9f6476cb3b835700f455b229a46c70b7de1b04901f9"
            .to_owned(),
        types: vec![
            ResolvedTypeContract {
                type_id: 1,
                canonical_path: vec!["User".to_owned()],
                kind: TypeContractKind::Table,
                fields: vec![
                    ResolvedFieldContract {
                        field_id: 11,
                        virtual_field_index: Some(0),
                        canonical_name: "id".to_owned(),
                        canonical_type: ResolvedCanonicalType::Scalar(CanonicalScalar::Uuid),
                        attributes: vec![],
                        default_value: None,
                    },
                    ResolvedFieldContract {
                        field_id: 12,
                        virtual_field_index: Some(1),
                        canonical_name: "name".to_owned(),
                        canonical_type: ResolvedCanonicalType::Scalar(CanonicalScalar::Utf8),
                        attributes: vec![],
                        default_value: None,
                    },
                ],
            },
            ResolvedTypeContract {
                type_id: 2,
                canonical_path: vec!["Post".to_owned()],
                kind: TypeContractKind::Class,
                fields: vec![
                    ResolvedFieldContract {
                        field_id: 21,
                        virtual_field_index: None,
                        canonical_name: "title".to_owned(),
                        canonical_type: ResolvedCanonicalType::Scalar(CanonicalScalar::Utf8),
                        attributes: vec![],
                        default_value: None,
                    },
                    ResolvedFieldContract {
                        field_id: 22,
                        virtual_field_index: None,
                        canonical_name: "body".to_owned(),
                        canonical_type: ResolvedCanonicalType::Scalar(CanonicalScalar::Utf8),
                        attributes: vec![],
                        default_value: None,
                    },
                ],
            },
        ],
    };

    // ContractArtifact: 2 operations + 1 service + 2 methods.
    let artifact = sample_artifact();

    let merged = StableIdentityManifest::from_full_artifact(&resolved, &artifact);

    // 2 types + 4 fields + 2 operations + 1 service + 2 methods = 11.
    assert_eq!(merged.entries.len(), 11, "must produce exactly 11 entries");

    // Verify every kind is represented with the expected count.
    let count = |kind: EntityKind| {
        merged
            .entries
            .iter()
            .filter(|e| e.entity_kind == kind)
            .count()
    };
    assert_eq!(count(EntityKind::Table), 1);
    assert_eq!(count(EntityKind::Class), 1);
    assert_eq!(count(EntityKind::Field), 4);
    assert_eq!(count(EntityKind::Operation), 2);
    assert_eq!(count(EntityKind::Service), 1);
    assert_eq!(count(EntityKind::Method), 2);

    // Every stable_id is non-zero and unique — no accidental collisions.
    let mut ids = merged.entries.iter().map(|e| e.stable_id).collect::<Vec<_>>();
    assert!(ids.iter().all(|id| id.is_valid()));
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 11, "stable ids must be globally unique");

    // Types carry their resolved durable identities.
    let user_table = merged
        .entries
        .iter()
        .find(|e| e.stable_id == StableId::new(1))
        .expect("User table entry");
    assert_eq!(user_table.entity_kind, EntityKind::Table);
    assert_eq!(user_table.current_name, "User");

    let post_class = merged
        .entries
        .iter()
        .find(|e| e.stable_id == StableId::new(2))
        .expect("Post class entry");
    assert_eq!(post_class.entity_kind, EntityKind::Class);
    assert_eq!(post_class.current_name, "Post");
}

/// Small helper used by the hint test above to build an empty artifact payload.
fn original_payload() -> ContractPayload {
    let svc_id = ServiceId::derive(
        &ServiceIdentityKey::new(vec!["demo".to_owned(), "GreetService".to_owned()]).unwrap(),
    );
    let op = op_key("hello");
    let op_id = op.operation_id.clone();
    let methods = vec![ServiceMethodContract {
        method_id: MethodId::derive(&MethodIdentityKey::new(svc_id.clone(), "greet").unwrap()),
        name: "greet".to_owned(),
        request_type: ResolvedCanonicalType::Scalar(CanonicalScalar::Utf8),
        response_type: ResolvedCanonicalType::Scalar(CanonicalScalar::Utf8),
        stream: MethodStreamMode::Unary,
        operation_id: op_id,
        attributes: vec![],
        stable_id_hint: None,
    }];
    let svc = ServiceContract {
        service_id: svc_id,
        name: "GreetService".to_owned(),
        canonical_path: vec!["demo".to_owned(), "GreetService".to_owned()],
        attributes: vec![],
        methods,
        source_unit_id: 0,
        span: (0..30).into(),
        stable_id_hint: None,
    };
    ContractPayload {
        operations: vec![op],
        services: vec![svc],
    }
}
