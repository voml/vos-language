//! The `file` descriptor contract: identity, metadata, read windows, EOF and
//! reference release, all without binding to a storage API.

use vos_types::{
    FILE_DESCRIPTOR_FORMAT_VERSION, FileDescriptor, FileManifestIdentity, FileMetadata,
    FileReadWindow, FileReference, FileReferenceState,
};

fn octets() -> Vec<u8> {
    (0..16).collect()
}

fn metadata(name: Option<&str>, media_type: Option<&str>) -> FileMetadata {
    FileMetadata {
        name: name.map(str::to_owned),
        media_type: media_type.map(str::to_owned),
    }
}

#[test]
fn describe_computes_the_manifest_identity_from_the_complete_octets() {
    let descriptor = FileDescriptor::describe(
        &octets(),
        metadata(Some("ledger.bin"), Some("application/octet-stream")),
    );
    assert_eq!(
        descriptor.identity.digest,
        "be45cb2605bf36bebde684841a28f0fd43c69850a3dce5fedba69928ee3a8991"
    );
    assert_eq!(descriptor.identity.octets, 16);
    assert_eq!(descriptor.octets(), 16);
    assert_eq!(descriptor.format_version, FILE_DESCRIPTOR_FORMAT_VERSION);
    assert_eq!(descriptor.validate(), Ok(()));
}

#[test]
fn metadata_never_enters_the_manifest_identity() {
    let named = FileDescriptor::describe(
        &octets(),
        metadata(Some("ledger.bin"), Some("application/octet-stream")),
    );
    let anonymous = FileDescriptor::describe(&octets(), FileMetadata::default());

    assert_eq!(
        named.identity, anonymous.identity,
        "renaming or relabelling a file must not change its content identity"
    );
    assert_ne!(
        named.metadata, anonymous.metadata,
        "name and media type stay separate from the content identity"
    );
    assert_ne!(
        named.to_json().unwrap(),
        anonymous.to_json().unwrap(),
        "the descriptor carries metadata next to the identity, not inside it"
    );
}

#[test]
fn descriptor_round_trips_through_a_strict_artifact() {
    let descriptor = FileDescriptor::describe(&octets(), FileMetadata::default());
    let json = descriptor.to_json().unwrap();
    assert_eq!(FileDescriptor::from_json(&json).unwrap(), descriptor);
    assert!(json.contains("vos-file-descriptor-v0"));
}

#[test]
fn full_and_range_reads_are_carried_by_the_descriptor() {
    let descriptor = FileDescriptor::describe(&octets(), FileMetadata::default());
    let reference = FileReference::live(descriptor.clone());

    assert_eq!(descriptor.full_window(), FileReadWindow::new(0, 16));
    assert_eq!(
        reference
            .load(reference.descriptor().full_window())
            .unwrap(),
        FileReadWindow::new(0, 16)
    );
    assert_eq!(
        reference.load(FileReadWindow::new(12, 8)).unwrap(),
        FileReadWindow::new(12, 4),
        "a range read that crosses end of file is a short read"
    );
    assert_eq!(
        reference.load(FileReadWindow::new(0, 100)).unwrap(),
        FileReadWindow::new(0, 16)
    );
}

#[test]
fn end_of_file_is_observable_at_and_beyond_the_last_octet() {
    let descriptor = FileDescriptor::describe(&octets(), FileMetadata::default());

    assert!(!descriptor.is_eof(15));
    assert!(
        descriptor.is_eof(16),
        "offset 16 is the first offset past the file"
    );
    assert!(descriptor.is_eof(20));
    assert_eq!(
        descriptor.window(FileReadWindow::new(16, 8)),
        FileReadWindow::new(16, 0)
    );
    assert_eq!(
        descriptor.window(FileReadWindow::new(20, 1)).end(),
        20,
        "an empty window stays at the requested offset"
    );
}

#[test]
fn zero_length_file_has_one_empty_window() {
    let descriptor = FileDescriptor::describe(&[], FileMetadata::default());
    assert_eq!(descriptor.full_window(), FileReadWindow::new(0, 0));
    assert!(descriptor.is_eof(0));
    assert_eq!(
        descriptor.window(FileReadWindow::new(0, 4)),
        FileReadWindow::new(0, 0)
    );
}

#[test]
fn released_reference_rejects_reads_and_release_is_idempotent() {
    let descriptor = FileDescriptor::describe(&octets(), FileMetadata::default());
    let mut reference = FileReference::live(descriptor.clone());
    assert_eq!(reference.state(), FileReferenceState::Live);

    reference.release();
    assert_eq!(reference.state(), FileReferenceState::Released);
    reference.release();
    assert_eq!(reference.state(), FileReferenceState::Released);

    let error = reference
        .load(FileReadWindow::new(0, 16))
        .expect_err("a released reference must not read");
    assert_eq!(error.code, "FIL005");
    assert_eq!(
        reference.descriptor().identity.octets,
        16,
        "the descriptor stays readable after release, only the reference is gone"
    );
}

#[test]
fn stale_version_and_malformed_digest_are_rejected() {
    let mut descriptor = FileDescriptor::describe(&octets(), FileMetadata::default());
    descriptor.format_version = "vos-file-descriptor-v1".to_owned();
    assert_eq!(descriptor.validate().unwrap_err().code, "FIL001");

    let mut identity = FileManifestIdentity::from_octets(&octets());
    identity.digest.make_ascii_uppercase();
    let descriptor = FileDescriptor::new(identity, FileMetadata::default());
    assert_eq!(descriptor.validate().unwrap_err().code, "FIL002");

    let short = FileManifestIdentity {
        digest: "be45cb26".to_owned(),
        octets: 16,
    };
    assert_eq!(
        FileDescriptor::new(short, FileMetadata::default())
            .validate()
            .unwrap_err()
            .code,
        "FIL002"
    );
}

#[test]
fn declared_but_empty_metadata_is_rejected() {
    for metadata in [
        FileMetadata {
            name: Some("  ".to_owned()),
            media_type: None,
        },
        FileMetadata {
            name: None,
            media_type: Some(String::new()),
        },
    ] {
        let descriptor =
            FileDescriptor::new(FileManifestIdentity::from_octets(&octets()), metadata);
        assert_eq!(descriptor.validate().unwrap_err().code, "FIL003");
    }
}

#[test]
fn unknown_descriptor_fields_and_shapes_are_rejected() {
    let json = r#"{
      "formatVersion": "vos-file-descriptor-v0",
      "identity": {
        "digest": "be45cb2605bf36bebde684841a28f0fd43c69850a3dce5fedba69928ee3a8991",
        "octets": 16
      },
      "metadata": {
        "name": "ledger.bin",
        "mediaType": "application/octet-stream"
      },
      "chunks": ["cas:sha256"]
    }"#;
    assert_eq!(FileDescriptor::from_json(json).unwrap_err().code, "FIL006");
}
