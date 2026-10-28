//! Canonical `bytes` value contract.
//!
//! The cases pin the two hard requirements: octets are never decoded as text on
//! any path, and the inline versus object-reference boundary belongs to the
//! storage implementation rather than to VOS.

use vos_types::{
    BYTES_VALUE_FORMAT_VERSION, BytesEncoding, BytesPlacement, BytesPlacementPolicy, BytesValue,
    BytesValueWire,
};

fn wire(encoding: &str, data: &str) -> BytesValueWire {
    BytesValueWire {
        format_version: BYTES_VALUE_FORMAT_VERSION.to_owned(),
        encoding: encoding.to_owned(),
        data: data.to_owned(),
    }
}

#[test]
fn empty_octet_sequence_round_trips() {
    let value = BytesValue::new(Vec::new());
    assert!(value.is_empty());
    assert_eq!(value.len(), 0);
    assert_eq!(value.to_wire(), wire("hex", ""));

    let decoded = BytesValue::from_json(&value.to_json().unwrap()).unwrap();
    assert_eq!(decoded, value);
}

#[test]
fn nul_octets_are_ordinary_octets() {
    let value = BytesValue::new(vec![0x00, 0x01, 0x00]);
    assert_eq!(value.len(), 3);
    assert_eq!(value.to_wire(), wire("hex", "000100"));
    assert_eq!(
        BytesValue::from_json(&value.to_json().unwrap()).unwrap(),
        value
    );
}

#[test]
fn octets_that_are_not_utf8_are_never_decoded() {
    let value = BytesValue::new(vec![0xff, 0xfe, 0x80]);
    assert!(
        std::str::from_utf8(value.as_octets()).is_err(),
        "the fixture octets are deliberately not UTF-8"
    );
    assert_eq!(value.to_wire(), wire("hex", "fffe80"));

    let json = value.to_json().unwrap();
    assert!(
        !json.contains('\u{fffd}'),
        "a lossy decode appeared in the artifact"
    );
    assert_eq!(
        BytesValue::from_json(&json).unwrap().into_octets(),
        vec![0xff, 0xfe, 0x80]
    );
}

#[test]
fn octets_that_would_look_like_utf8_stay_two_octets() {
    let value = BytesValue::new(vec![0xc3, 0xa9]);
    assert_eq!(
        value.len(),
        2,
        "a UTF-8 decode would collapse this into one character"
    );
    assert_eq!(value.to_wire(), wire("hex", "c3a9"));
    assert_eq!(
        BytesValue::from_json(&value.to_json().unwrap())
            .unwrap()
            .as_octets(),
        &[0xc3, 0xa9]
    );
}

#[test]
fn a_large_octet_sequence_round_trips_without_truncation() {
    let octets = (0..8192)
        .map(|index| (index % 256) as u8)
        .collect::<Vec<u8>>();
    let value = BytesValue::new(octets.clone());
    assert_eq!(value.len(), 8192);

    let json = value.to_json().unwrap();
    assert_eq!(value.to_wire().data.len(), 8192 * 2);
    assert_eq!(BytesValue::from_json(&json).unwrap().into_octets(), octets);
}

#[test]
fn odd_hex_length_is_rejected() {
    let error = BytesValue::from_wire(wire("hex", "000")).unwrap_err();
    assert_eq!(error.code, "BYT003");
}

#[test]
fn non_hexadecimal_data_is_rejected() {
    let error = BytesValue::from_wire(wire("hex", "00zz")).unwrap_err();
    assert_eq!(error.code, "BYT004");
}

#[test]
fn uppercase_hex_is_not_a_second_spelling() {
    let error = BytesValue::from_wire(wire("hex", "AABB")).unwrap_err();
    assert_eq!(error.code, "BYT004");
}

#[test]
fn unsupported_encoding_is_rejected() {
    let error = BytesValue::from_wire(wire("base64", "AA==")).unwrap_err();
    assert_eq!(error.code, "BYT002");
    assert_eq!(BytesEncoding::from_name("base64"), None);
    assert_eq!(BytesEncoding::from_name("hex"), Some(BytesEncoding::Hex));
    assert_eq!(BytesEncoding::Hex.as_str(), "hex");
}

#[test]
fn stale_format_version_is_rejected() {
    let mut value = wire("hex", "0001");
    value.format_version = "vos-bytes-value-v1".to_owned();
    assert_eq!(BytesValue::from_wire(value).unwrap_err().code, "BYT001");
}

#[test]
fn unknown_wire_fields_are_rejected() {
    let input = r#"{
      "formatVersion": "vos-bytes-value-v0",
      "encoding": "hex",
      "data": "0001",
      "objectRef": "cas:sha256"
    }"#;
    assert_eq!(BytesValue::from_json(input).unwrap_err().code, "BYT005");
}

struct HostDeclaringItsOwnBoundary {
    inline_limit_octets: usize,
}

impl BytesPlacementPolicy for HostDeclaringItsOwnBoundary {
    fn placement_for(&self, octets: &[u8]) -> BytesPlacement {
        if octets.len() <= self.inline_limit_octets {
            BytesPlacement::Inline
        } else {
            BytesPlacement::ObjectReference
        }
    }
}

#[test]
fn placement_is_declared_by_the_storage_implementation() {
    let value = BytesValue::new(vec![0x01; 17]);
    let small_inline_budget = HostDeclaringItsOwnBoundary {
        inline_limit_octets: 16,
    };
    let large_inline_budget = HostDeclaringItsOwnBoundary {
        inline_limit_octets: 1024,
    };

    assert_eq!(
        value.placement(&small_inline_budget),
        BytesPlacement::ObjectReference
    );
    assert_eq!(
        value.placement(&large_inline_budget),
        BytesPlacement::Inline,
        "the same value is placed differently by a different host declaration"
    );
}
