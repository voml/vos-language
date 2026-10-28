//! Cross-document conformance fixtures for VOS multi-file resolution.
//!
//! The `multi_document` fixture tree is the consumable shape of the feature: a
//! real `using` graph on disk, one document per type family, and one service
//! document that binds an operation declared in a sibling document. Every
//! negative case is a fixture too, so an unresolved import, a `using` cycle, a
//! dangling type reference, a duplicate type path and a duplicate operation are
//! asserted by exact diagnostic code and message instead of "something failed".

use std::fs;
use std::path::{Path, PathBuf};

use vos::vfs::{FsVfs, VfsErrorKind};
use vos::{
    CanonicalType, ContractArtifact, ContractFormat, ContractPayload, ContractProducer, DeclKind,
    IdentityBoundProjection, IdentityManifest, MethodStreamMode, ResolvedCanonicalType,
    ResolvedContract, ServiceContractDecl, ServiceMethodDecl, bind_identity,
    build_operation_contracts, build_service_contracts, parse_oak,
    parse_operation_semantics_from_attributes, resolve_contract_from_units, resolve_using_tree,
};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../documentation/fixtures/contracts/multi_document")
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn canonical(path: &Path) -> PathBuf {
    path.canonicalize()
        .unwrap_or_else(|error| panic!("canonicalize {}: {error}", path.display()))
}

/// Parses one fixture document and binds it to its reviewed identity manifest.
fn bound(source: &str, manifest: &str) -> IdentityBoundProjection {
    let source_path = fixtures().join(source);
    let projection = parse_oak(&read(&source_path))
        .expect("Oak parses the fixture document")
        .project_schema()
        .expect("the fixture document projects into a schema");
    let manifest = IdentityManifest::from_json(&read(&fixtures().join(manifest)))
        .expect("the fixture manifest is a strict identity manifest");
    bind_identity(&projection, &manifest)
        .unwrap_or_else(|diagnostics| panic!("bind {source}: {diagnostics:?}"))
}

fn vault_contract() -> ResolvedContract {
    let ledger = bound("vault/ledger.vos", "vault/ledger.manifest.json");
    let account = bound("vault/account.vos", "vault/account.manifest.json");
    resolve_contract_from_units(&[&ledger, &account])
        .unwrap_or_else(|diagnostics| panic!("resolve the vault contract: {diagnostics:?}"))
}

#[test]
fn using_tree_loads_every_document_of_the_vault_fixture() {
    let entry_path = fixtures().join("vault/workspace.vos");
    let table = resolve_using_tree(&entry_path, &FsVfs::new())
        .expect("the fixture `using` graph resolves without a cycle");

    assert_eq!(
        table.sources.len(),
        4,
        "the workspace entry plus service, ledger and account documents must all load"
    );

    let entry = &table.sources[&canonical(&entry_path)];
    assert_eq!(entry.namespace, ["demo", "vault"]);
    assert_eq!(
        entry.using_paths,
        vec![
            "./account.vos".to_owned(),
            "./ledger.vos".to_owned(),
            "./service.vos".to_owned()
        ],
        "every `using` path must survive the collect pass"
    );
    assert_eq!(entry.declarations.len(), 1, "the entry only declares a namespace");
    assert!(matches!(
        &entry.declarations[0],
        DeclKind::Namespace { path } if path.join("::") == "demo::vault"
    ));

    let service = &table.sources[&canonical(&fixtures().join("vault/service.vos"))];
    assert!(matches!(
        &service.declarations[1],
        DeclKind::Service { name } if name == "Vault"
    ));
    assert!(matches!(
        &service.declarations[2],
        DeclKind::Operation { name, .. } if name == "fetch"
    ));

    let ledger = &table.sources[&canonical(&fixtures().join("vault/ledger.vos"))];
    assert!(matches!(
        &ledger.declarations[1],
        DeclKind::Type { name, .. } if name == "Ledger"
    ));
    assert!(matches!(
        &ledger.declarations[2],
        DeclKind::Operation { name, .. } if name == "record"
    ));

    let account = &table.sources[&canonical(&fixtures().join("vault/account.vos"))];
    assert!(matches!(
        &account.declarations[1],
        DeclKind::Type { name, .. } if name == "Account"
    ));
    assert!(
        account.using_paths.is_empty(),
        "the leaf document must not import anything"
    );
}

#[test]
fn cross_document_type_reference_resolves_to_the_owning_unit() {
    let forward = vault_contract();
    forward
        .validate()
        .expect("a cross-document resolved contract validates");
    assert_eq!(forward.types.len(), 2);

    let ledger = forward
        .types
        .iter()
        .find(|item| item.canonical_path == ["demo", "vault", "Ledger"])
        .expect("Ledger is resolved from its own document");
    let owner = ledger
        .fields
        .iter()
        .find(|field| field.canonical_name == "owner")
        .expect("the owner field survives resolution");
    assert_eq!(
        owner.canonical_type,
        ResolvedCanonicalType::User {
            path: vec![
                "demo".to_owned(),
                "vault".to_owned(),
                "Account".to_owned()
            ],
            type_id: 1,
        },
        "a type declared in another document resolves to that document's durable identity"
    );

    let reversed = {
        let ledger = bound("vault/ledger.vos", "vault/ledger.manifest.json");
        let account = bound("vault/account.vos", "vault/account.manifest.json");
        resolve_contract_from_units(&[&account, &ledger]).expect("the reversed unit order resolves")
    };
    assert_eq!(
        forward.schema_fingerprint, reversed.schema_fingerprint,
        "source-unit order must not change the schema fingerprint"
    );
}

#[test]
fn service_binds_an_operation_declared_in_another_document() {
    let types = vault_contract().types;
    let service_input = parse_oak(&read(&fixtures().join("vault/service.vos")))
        .expect("Oak parses the service document");
    let ledger_input = parse_oak(&read(&fixtures().join("vault/ledger.vos")))
        .expect("Oak parses the ledger document");

    let service_operations = service_input
        .project_operations()
        .expect("the service document projects its operation");
    let ledger_operations = ledger_input
        .project_operations()
        .expect("the ledger document projects its operation");
    assert_eq!(service_operations.len(), 1);
    assert_eq!(ledger_operations.len(), 1);
    assert_eq!(service_operations[0].name, "fetch");
    assert_eq!(ledger_operations[0].name, "record");

    let semantics = service_operations
        .iter()
        .chain(ledger_operations.iter())
        .map(|operation| {
            parse_operation_semantics_from_attributes(operation)
                .expect("both fixtures declare their effect")
        })
        .collect::<Vec<_>>();
    let operations = build_operation_contracts(
        &[(0, &service_operations), (1, &ledger_operations)],
        &semantics,
        &types,
        None,
    )
    .expect("operations from two documents build together");
    assert_eq!(operations.len(), 2);

    let record = operations
        .iter()
        .find(|operation| operation.name == "record")
        .expect("record has a contract");
    assert_eq!(
        record.source_unit_id, 1,
        "record keeps the provenance of the document that declared it"
    );
    let fetch_id = operations
        .iter()
        .find(|operation| operation.name == "fetch")
        .expect("fetch has a contract")
        .operation_id
        .clone();

    let services = service_input
        .project_services()
        .expect("the service document projects its service");
    assert_eq!(services.len(), 1);
    let declaration = ServiceContractDecl {
        service_id: services[0].service_id.clone(),
        methods: vec![
            ServiceMethodDecl {
                name: "fetch".to_owned(),
                request_type: CanonicalType::Named(vec!["uuid".to_owned()]),
                response_type: CanonicalType::Optional(Box::new(CanonicalType::Named(vec![
                    "Account".to_owned(),
                ]))),
                stream: MethodStreamMode::Unary,
                operation_id: fetch_id,
            },
            ServiceMethodDecl {
                name: "record".to_owned(),
                request_type: CanonicalType::Named(vec!["Account".to_owned()]),
                response_type: CanonicalType::Named(vec!["Ledger".to_owned()]),
                stream: MethodStreamMode::Unary,
                operation_id: record.operation_id.clone(),
            },
        ],
    };
    let contracts = build_service_contracts(&[(0, &services)], &[declaration], &operations, &types, None)
        .expect("the service binds an operation declared in a sibling document");

    let artifact = ContractArtifact::seal(
        ContractFormat::Iris,
        ContractProducer::new("fixture", "0"),
        ContractPayload {
            operations,
            services: contracts,
        },
    );
    artifact
        .validate()
        .expect("the cross-document artifact validates");
    assert_eq!(artifact.payload.operations.len(), 2);
    assert_eq!(artifact.payload.services.len(), 1);

    let bound_record = artifact.payload.services[0]
        .methods
        .iter()
        .find(|method| method.name == "record")
        .expect("record is bound as a method")
        .operation_id
        .clone();
    assert!(
        artifact
            .payload
            .operations
            .iter()
            .any(|operation| operation.operation_id == bound_record && operation.source_unit_id == 1),
        "the method binding must point at the operation owned by the other document"
    );
}

#[test]
fn unresolved_using_path_is_reported_as_not_found() {
    let (_partial, diagnostics) =
        resolve_using_tree(&fixtures().join("unresolved/entry.vos"), &FsVfs::new())
            .expect_err("an unresolved `using` path must fail");

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].kind, VfsErrorKind::NotFound);
    assert!(
        diagnostics[0].message.contains("missing.vos"),
        "the diagnostic must name the unresolved document: {}",
        diagnostics[0].message
    );
}

#[test]
fn circular_using_chain_names_both_documents() {
    let (partial, diagnostics) = resolve_using_tree(&fixtures().join("cycle/a.vos"), &FsVfs::new())
        .expect_err("a `using` cycle must fail");

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].kind, VfsErrorKind::Circular);
    assert!(
        diagnostics[0].message.contains("a.vos") && diagnostics[0].message.contains("b.vos"),
        "the cycle must name every document on the stack: {}",
        diagnostics[0].message
    );
    assert_eq!(
        partial.sources.len(),
        2,
        "both documents are still collected before the cycle is reported"
    );
}

#[test]
fn dangling_type_reference_is_diagnosed_as_res001() {
    let reference = bound("dangling/reference.vos", "dangling/reference.manifest.json");
    let diagnostics = resolve_contract_from_units(&[&reference])
        .expect_err("a reference to a type no document declares must fail");

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "RES001");
    assert_eq!(diagnostics[0].message, "unknown type Account");
}

#[test]
fn duplicate_type_path_across_documents_is_diagnosed_as_res004() {
    let first = bound("duplicate/first.vos", "duplicate/first.manifest.json");
    let second = bound("duplicate/second.vos", "duplicate/second.manifest.json");

    resolve_contract_from_units(&[&first])
        .expect("one document alone resolves")
        .validate()
        .expect("one document alone validates");

    let diagnostics = resolve_contract_from_units(&[&first, &second])
        .expect_err("two documents declaring one type path must fail");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "RES004");
    assert_eq!(diagnostics[0].message, "duplicate type path across source units");
}

#[test]
fn duplicate_type_and_field_ids_across_documents_are_diagnosed() {
    let first = bound("duplicate/first.vos", "duplicate/first.manifest.json");
    let other = bound("duplicate/other.vos", "duplicate/other.manifest.json");

    let diagnostics = resolve_contract_from_units(&[&first, &other])
        .expect_err("two documents reusing one durable type id must fail");
    assert_eq!(diagnostics.len(), 2);
    assert!(
        diagnostics
            .iter()
            .any(|item| item.code == "RES007"
                && item.message == "duplicate type ID across source units"),
        "the duplicate type id must be reported: {diagnostics:?}"
    );
    assert!(
        diagnostics
            .iter()
            .any(|item| item.code == "RES006"
                && item.message == "duplicate field ID across source units"),
        "the duplicate field id must be reported: {diagnostics:?}"
    );
}

#[test]
fn operation_declared_in_two_documents_is_diagnosed_as_opc003() {
    let first = parse_oak(&read(&fixtures().join("duplicate/fetch_first.vos")))
        .expect("Oak parses the first operation document");
    let second = parse_oak(&read(&fixtures().join("duplicate/fetch_second.vos")))
        .expect("Oak parses the second operation document");
    let first_operations = first.project_operations().expect("fetch projects");
    let second_operations = second.project_operations().expect("fetch projects");
    assert_eq!(
        first_operations[0].operation_id, second_operations[0].operation_id,
        "the fixture duplicates exactly one operation across two documents"
    );

    let semantics = first_operations
        .iter()
        .map(|operation| {
            parse_operation_semantics_from_attributes(operation)
                .expect("the fixture declares its effect")
        })
        .collect::<Vec<_>>();
    let diagnostics = build_operation_contracts(
        &[(0, &first_operations), (1, &second_operations)],
        &semantics,
        &[],
        None,
    )
    .expect_err("one operation declared in two documents must fail");

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "OPC003");
    assert!(
        diagnostics[0]
            .message
            .contains(first_operations[0].operation_id.as_str()),
        "the diagnostic must name the duplicated operation: {}",
        diagnostics[0].message
    );
}
