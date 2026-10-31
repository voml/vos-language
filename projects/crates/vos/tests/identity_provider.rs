//! Conformance fixtures for the durable identity manifest provider.
//!
//! Every requirement of the identity contract is asserted here as a fixture:
//! first-round allocation follows a deterministic canonical order (never
//! source order), reordering declarations moves no identity and never changes
//! the schema fingerprint, unrelated additions and removals keep active IDs
//! while retired IDs are never reused, explicit renames preserve durable
//! identity while the fingerprint follows the canonical path, externally
//! authored manifests with unknown or conflicting assignments fail with an
//! exact `IMP` diagnostic code, and persisted provider state survives
//! compile cycles while a corrupt state file fails explicitly.

use core::range::Range;
use std::fs;
use std::io;
use std::path::PathBuf;

use vos::{
    CANONICALIZATION_VERSION, CONTRACT_FORMAT_VERSION, CanonicalType, ContractEnvelope,
    FieldContract, FieldIdentity, IDENTITY_MANIFEST_VERSION, IdentityChange, IdentityManifest,
    IdentityManifestProvider, LANGUAGE_VERSION, PROJECTION_STAGE, SchemaProjection, SourceUnit,
    TypeContract, TypeContractKind, TypeIdentity, bind_identity, compare_identity, parse_oak,
    resolve_contract_from_units,
};

fn project(source: &str) -> SchemaProjection {
    parse_oak(source)
        .expect("Oak parses the fixture source")
        .project_schema()
        .expect("VOS projects the fixture source")
}

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "vos-identity-provider-{tag}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("the temp fixture directory is created");
    dir
}

fn type_id_of(manifest: &IdentityManifest, path: &[&str]) -> u64 {
    manifest
        .types
        .iter()
        .find(|item| {
            item.canonical_path
                .iter()
                .map(String::as_str)
                .eq(path.iter().copied())
        })
        .unwrap_or_else(|| panic!("{} is present in the manifest", path.join("::")))
        .type_id
}

fn field_id_of(manifest: &IdentityManifest, path: &[&str], field: &str) -> u64 {
    let identity = manifest
        .types
        .iter()
        .find(|item| {
            item.canonical_path
                .iter()
                .map(String::as_str)
                .eq(path.iter().copied())
        })
        .unwrap_or_else(|| panic!("{} is present in the manifest", path.join("::")));
    identity
        .fields
        .iter()
        .find(|item| item.canonical_name == field)
        .unwrap_or_else(|| panic!("{field} is present in {}", path.join("::")))
        .field_id
}

#[test]
fn first_round_allocates_by_canonical_order_not_source_order() {
    let mut provider = IdentityManifestProvider::empty();
    let projection = project("namespace demo\ntable Zed { id: uuid, }\ntable Alpha { id: uuid, }");
    let manifest = provider
        .ensure(&projection)
        .expect("the first round allocates");

    assert_eq!(type_id_of(&manifest, &["demo", "Alpha"]), 1);
    assert_eq!(type_id_of(&manifest, &["demo", "Zed"]), 2);

    let bound = bind_identity(&projection, &manifest).expect("the manifest binds");
    let contract = resolve_contract_from_units(&[&bound]).expect("the contract resolves");
    contract
        .validate()
        .expect("the resolved contract validates");
    assert_eq!(contract.types.len(), 2);
}

#[test]
fn reordering_declarations_never_moves_identities_or_fingerprint() {
    let mut provider = IdentityManifestProvider::empty();
    let first = project(
        "namespace demo\ntable Beta { name: utf8, id: uuid, }\ntable Alpha { email: utf8?, id: uuid, }",
    );
    let manifest_before = provider.ensure(&first).expect("the first round allocates");
    let bound_before = bind_identity(&first, &manifest_before).expect("the manifest binds");
    let fingerprint_before = resolve_contract_from_units(&[&bound_before])
        .expect("the contract resolves")
        .schema_fingerprint;

    let second = project(
        "namespace demo\ntable Alpha { id: uuid, email: utf8?, }\ntable Beta { id: uuid, name: utf8, }",
    );
    let manifest_after = provider
        .ensure(&second)
        .expect("the reordered round reconciles");
    assert_eq!(
        manifest_before, manifest_after,
        "reordering type and field declarations must not move any identity"
    );

    let bound_after = bind_identity(&second, &manifest_after).expect("the manifest binds");
    let fingerprint_after = resolve_contract_from_units(&[&bound_after])
        .expect("the contract resolves")
        .schema_fingerprint;
    assert_eq!(
        fingerprint_before, fingerprint_after,
        "pure reordering must not change the canonical schema fingerprint"
    );
}

#[test]
fn unrelated_additions_and_removals_keep_and_retire_durable_ids() {
    let mut provider = IdentityManifestProvider::empty();
    let base = "namespace demo\ntable Alpha { id: uuid, }\ntable Beta { id: uuid, }";
    let first = provider
        .ensure(&project(base))
        .expect("the first round allocates");
    let alpha_id = type_id_of(&first, &["demo", "Alpha"]);
    let beta_id = type_id_of(&first, &["demo", "Beta"]);

    let second = provider
        .ensure(&project(
            "namespace demo\ntable Alpha { id: uuid, }\ntable Beta { id: uuid, }\ntable Gamma { id: uuid, }",
        ))
        .expect("an unrelated type is added");
    assert_eq!(type_id_of(&second, &["demo", "Alpha"]), alpha_id);
    assert_eq!(type_id_of(&second, &["demo", "Beta"]), beta_id);
    let gamma_id = type_id_of(&second, &["demo", "Gamma"]);

    provider
        .ensure(&project(base))
        .expect("the unrelated type is removed");
    assert!(
        provider.retired_type_ids().contains(&gamma_id),
        "a removed type must be retired"
    );

    provider
        .ensure(&project(
            "namespace demo\ntable Alpha { id: uuid, }\ntable Beta { id: uuid, }\ntable Delta { id: uuid, }",
        ))
        .expect("a fresh type is added after the retirement");
    let delta_id = provider
        .type_id(&["demo".to_owned(), "Delta".to_owned()])
        .expect("Delta has a durable identity");
    assert_ne!(delta_id, gamma_id, "a retired type ID is never reused");
    assert_eq!(delta_id, gamma_id + 1, "allocation stays monotonic");

    let mut field_provider = IdentityManifestProvider::empty();
    let with_email = field_provider
        .ensure(&project(
            "namespace demo\ntable Alpha { id: uuid, email: utf8?, }",
        ))
        .expect("the first round allocates");
    let email_field_id = field_id_of(&with_email, &["demo", "Alpha"], "email");
    field_provider
        .ensure(&project("namespace demo\ntable Alpha { id: uuid, }"))
        .expect("the field is removed");
    assert!(field_provider.retired_field_ids().contains(&email_field_id));
    field_provider
        .ensure(&project(
            "namespace demo\ntable Alpha { id: uuid, phone: utf8?, }",
        ))
        .expect("a fresh field is added");
    assert_ne!(
        field_provider
            .field_id(&["demo".to_owned(), "Alpha".to_owned()], "phone")
            .expect("phone has a durable identity"),
        email_field_id,
        "a retired field ID is never reused"
    );
}

#[test]
fn explicit_renames_preserve_identity_and_report_events() {
    let mut provider = IdentityManifestProvider::empty();
    let before = project("namespace demo\ntable Alpha { id: uuid, email: utf8?, }");
    let manifest_before = provider.ensure(&before).expect("the first round allocates");
    let bound_before = bind_identity(&before, &manifest_before).expect("the manifest binds");
    let alpha_id = type_id_of(&manifest_before, &["demo", "Alpha"]);
    let email_field_id = field_id_of(&manifest_before, &["demo", "Alpha"], "email");

    provider
        .rename_type(
            &["demo".to_owned(), "Alpha".to_owned()],
            &["demo".to_owned(), "Renamed".to_owned()],
        )
        .expect("the declared type rename applies");
    provider
        .rename_field(
            &["demo".to_owned(), "Renamed".to_owned()],
            "email",
            "contact",
        )
        .expect("the declared field rename applies");

    let after = project("namespace demo\ntable Renamed { contact: utf8?, id: uuid, }");
    let manifest_after = provider
        .ensure(&after)
        .expect("the renamed round reconciles");
    assert_eq!(type_id_of(&manifest_after, &["demo", "Renamed"]), alpha_id);
    assert_eq!(
        field_id_of(&manifest_after, &["demo", "Renamed"], "contact"),
        email_field_id
    );

    let bound_after = bind_identity(&after, &manifest_after).expect("the manifest binds");
    let evolution = compare_identity(&bound_before, &bound_after).expect("the evolution compares");
    assert!(evolution.changes.contains(&IdentityChange::TypeRenamed {
        type_id: alpha_id,
        from: vec!["demo".to_owned(), "Alpha".to_owned()],
        to: vec!["demo".to_owned(), "Renamed".to_owned()],
    }));
    assert!(evolution.changes.contains(&IdentityChange::FieldRenamed {
        type_id: alpha_id,
        field_id: email_field_id,
        from: "email".to_owned(),
        to: "contact".to_owned(),
    }));

    let fingerprint_before = resolve_contract_from_units(&[&bound_before])
        .expect("the contract resolves")
        .schema_fingerprint;
    let fingerprint_after = resolve_contract_from_units(&[&bound_after])
        .expect("the contract resolves")
        .schema_fingerprint;
    assert_ne!(
        fingerprint_before, fingerprint_after,
        "the canonical path is semantic, so a rename regenerates the fingerprint"
    );
}

fn setup_provider_with_retirements() -> IdentityManifestProvider {
    let mut provider = IdentityManifestProvider::empty();
    provider
        .ensure(&project(
            "namespace demo\ntable Alpha { id: uuid, email: utf8?, }\ntable Beta { name: utf8, }",
        ))
        .expect("the first round allocates");
    provider
        .ensure(&project("namespace demo\ntable Alpha { id: uuid, }"))
        .expect("Beta and email are removed");
    provider
}

fn manifest_with(types: Vec<TypeIdentity>) -> IdentityManifest {
    IdentityManifest {
        format_version: IDENTITY_MANIFEST_VERSION.to_owned(),
        types,
    }
}

fn alpha_identity(type_id: u64, fields: Vec<FieldIdentity>) -> TypeIdentity {
    TypeIdentity {
        canonical_path: vec!["demo".to_owned(), "Alpha".to_owned()],
        type_id,
        kind: TypeContractKind::Table,
        fields,
    }
}

fn field_identity(name: &str, field_id: u64, slot: Option<u32>) -> FieldIdentity {
    FieldIdentity {
        canonical_name: name.to_owned(),
        field_id,
        virtual_field_index: slot,
    }
}

#[test]
fn apply_manifest_fails_explicitly_on_unknown_or_conflicting_ids() {
    let mut provider = setup_provider_with_retirements();
    let snapshot = provider.clone();
    let alpha_id = provider
        .type_id(&["demo".to_owned(), "Alpha".to_owned()])
        .expect("Alpha is active");
    let id_field_id = provider
        .field_id(&["demo".to_owned(), "Alpha".to_owned()], "id")
        .expect("id is active");
    let retired_type_id = provider.retired_type_ids()[0];
    let retired_field_id = provider.retired_field_ids()[0];

    let wrong_version = IdentityManifest {
        format_version: "bogus".to_owned(),
        types: Vec::new(),
    };
    let diagnostics = provider.apply_manifest(&wrong_version).unwrap_err();
    assert_eq!(diagnostics[0].code, "IMP004");

    let cases: Vec<(IdentityManifest, &str)> = vec![
        (
            manifest_with(vec![alpha_identity(
                alpha_id + 98,
                vec![field_identity("id", id_field_id, Some(1))],
            )]),
            "IMP005",
        ),
        (
            manifest_with(vec![
                alpha_identity(alpha_id, vec![field_identity("id", id_field_id, Some(0))]),
                TypeIdentity {
                    canonical_path: vec!["demo".to_owned(), "Gamma".to_owned()],
                    type_id: retired_type_id,
                    kind: TypeContractKind::Table,
                    fields: vec![field_identity("id", 90, Some(0))],
                },
            ]),
            "IMP008",
        ),
        (
            manifest_with(vec![
                alpha_identity(alpha_id, vec![field_identity("id", id_field_id, Some(0))]),
                TypeIdentity {
                    canonical_path: vec!["demo".to_owned(), "Gamma".to_owned()],
                    type_id: alpha_id,
                    kind: TypeContractKind::Table,
                    fields: vec![field_identity("id", 90, Some(0))],
                },
            ]),
            "IMP008",
        ),
        (
            manifest_with(vec![alpha_identity(
                alpha_id,
                vec![
                    field_identity("id", id_field_id, Some(0)),
                    field_identity("other", retired_field_id, Some(5)),
                ],
            )]),
            "IMP009",
        ),
        (
            manifest_with(vec![alpha_identity(
                alpha_id,
                vec![
                    field_identity("id", id_field_id, Some(0)),
                    field_identity("other", 90, Some(0)),
                ],
            )]),
            "IMP010",
        ),
        (
            manifest_with(vec![TypeIdentity {
                canonical_path: vec!["demo".to_owned(), "Zero".to_owned()],
                type_id: 0,
                kind: TypeContractKind::Table,
                fields: Vec::new(),
            }]),
            "IMP011",
        ),
    ];
    for (manifest, code) in cases {
        let diagnostics = provider.apply_manifest(&manifest).unwrap_err();
        assert!(
            diagnostics.iter().any(|item| item.code == code),
            "expected {code} from {diagnostics:?}"
        );
        assert_eq!(
            provider, snapshot,
            "a rejected manifest must not mutate the provider"
        );
    }
}

#[test]
fn apply_manifest_adopts_agreeing_assignments() {
    let mut provider = setup_provider_with_retirements();
    let alpha_id = provider
        .type_id(&["demo".to_owned(), "Alpha".to_owned()])
        .expect("Alpha is active");
    let id_field_id = provider
        .field_id(&["demo".to_owned(), "Alpha".to_owned()], "id")
        .expect("id is active");
    let adopted = manifest_with(vec![
        alpha_identity(alpha_id, vec![field_identity("id", id_field_id, Some(1))]),
        TypeIdentity {
            canonical_path: vec!["demo".to_owned(), "Gamma".to_owned()],
            type_id: alpha_id + 2,
            kind: TypeContractKind::Table,
            fields: vec![field_identity("id", id_field_id + 2, Some(0))],
        },
    ]);
    provider
        .apply_manifest(&adopted)
        .expect("the manifest is adopted");
    assert_eq!(
        provider.type_id(&["demo".to_owned(), "Gamma".to_owned()]),
        Some(alpha_id + 2)
    );

    let reconciled = provider
        .ensure(&project(
            "namespace demo\ntable Alpha { id: uuid, }\ntable Gamma { id: uuid, }",
        ))
        .expect("the adopted state reconciles");
    assert_eq!(reconciled, adopted);
}

#[test]
fn duplicate_and_empty_canonical_paths_are_rejected() {
    let mut provider = IdentityManifestProvider::empty();
    let duplicate = project("table Dup { id: uuid, }");
    let diagnostics = provider
        .ensure_units(&[&duplicate, &duplicate])
        .expect_err("the same canonical path cannot be allocated twice");
    assert_eq!(diagnostics[0].code, "IMP003");

    let empty_path = SchemaProjection {
        envelope: ContractEnvelope {
            stage: PROJECTION_STAGE.to_owned(),
            contract_format_version: CONTRACT_FORMAT_VERSION.to_owned(),
            language_version: LANGUAGE_VERSION.to_owned(),
            canonicalization_version: CANONICALIZATION_VERSION.to_owned(),
            schema_fingerprint: None,
            source_units: vec![SourceUnit {
                source_unit_id: 0,
                source: String::new(),
            }],
        },
        types: vec![TypeContract {
            canonical_path: Vec::new(),
            kind: TypeContractKind::Table,
            fields: vec![FieldContract {
                canonical_name: "id".to_owned(),
                canonical_type: CanonicalType::Named(vec!["uuid".to_owned()]),
                attributes: Vec::new(),
                default_value: None,
                span: Range { start: 0, end: 2 },
            }],
            span: Range { start: 0, end: 3 },
        }],
    };
    let diagnostics = provider
        .ensure(&empty_path)
        .expect_err("an empty canonical path is rejected");
    assert_eq!(diagnostics[0].code, "IMP001");
}

#[test]
fn provider_state_survives_compile_cycles() {
    let dir = temp_dir("cycle");
    let first_source = "namespace demo\ntable Alpha { id: uuid, }";
    let mut provider =
        IdentityManifestProvider::load_or_create(&dir).expect("a missing state starts empty");
    let first_manifest = provider
        .ensure(&project(first_source))
        .expect("the first cycle allocates");
    provider.save(&dir).expect("the state persists");

    let reloaded =
        IdentityManifestProvider::load_or_create(&dir).expect("the persisted state loads");
    assert_eq!(reloaded.manifest(), first_manifest);

    let mut provider = reloaded;
    let second_manifest = provider
        .ensure(&project(
            "namespace demo\ntable Alpha { id: uuid, }\ntable Beta { id: uuid, }",
        ))
        .expect("the second cycle reconciles");
    assert_eq!(type_id_of(&second_manifest, &["demo", "Alpha"]), 1);
    assert_eq!(type_id_of(&second_manifest, &["demo", "Beta"]), 2);
    provider.save(&dir).expect("the second cycle persists");

    fs::write(IdentityManifestProvider::provider_path(&dir), "not json")
        .expect("the state file is corrupted");
    let error = IdentityManifestProvider::load_or_create(&dir).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);

    fs::remove_dir_all(&dir).expect("the temp fixture directory is removed");
}
