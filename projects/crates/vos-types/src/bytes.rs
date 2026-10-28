//! Canonical value model of the `bytes` scalar.
//!
//! `bytes` is an arbitrary octet sequence. It is never text: no stage of VOS
//! decodes it as UTF-8, `utf16` or any host string, and no stage re-encodes it
//! through a lossy character buffer. A value is carried as [`BytesValue`] over
//! `Vec<u8>` from the moment a host reads it until a host stores it.
//!
//! The artifact wire form is [`BytesValueWire`]: a version tag, an explicit
//! [`BytesEncoding`] and the encoded octets. Only lowercase hexadecimal is
//! accepted, so one octet sequence has exactly one wire spelling and a
//! round-trip is stable.
//!
//! Whether a storage implementation keeps a value inline or as a
//! content-addressed object reference is a **physical** decision. VOS carries no
//! octet threshold and no contract field depends on one. A host declares its own
//! boundary by implementing [`BytesPlacementPolicy`] and asking
//! [`BytesValue::placement`].

use serde::{Deserialize, Serialize};

use crate::ArtifactError;

/// Version of the strict `bytes` value wire shape.
pub const BYTES_VALUE_FORMAT_VERSION: &str = "vos-bytes-value-v0";

const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

/// Wire encoding of a `bytes` value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BytesEncoding {
    /// Lowercase hexadecimal, two characters per octet.
    Hex,
}

impl BytesEncoding {
    /// Every encoding accepted by the wire contract.
    pub const ALL: [Self; 1] = [Self::Hex];

    /// Returns the canonical wire spelling of this encoding.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Hex => "hex",
        }
    }

    /// Reads an encoding by its exact wire spelling.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|encoding| encoding.as_str() == name)
    }
}

/// Where a storage implementation keeps a `bytes` value.
///
/// VOS names the two placements so a host, an artifact reader and a test can
/// agree on the vocabulary, but it never decides between them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BytesPlacement {
    /// The octets live inside the owning record.
    Inline,
    /// The octets live in a content-addressed object the owning record references.
    ObjectReference,
}

/// A storage implementation's own inline versus object-reference boundary.
///
/// Implementations return the placement their physical layout uses for the
/// given octets. There is deliberately no threshold in VOS: a host whose
/// `.yydb` file, network object store or in-memory arena has different limits
/// implements this trait differently and no contract value changes.
pub trait BytesPlacementPolicy {
    /// Declares where this implementation stores the given octets.
    fn placement_for(&self, octets: &[u8]) -> BytesPlacement;
}

/// Strict wire form of a `bytes` value.
///
/// The fields are exactly the artifact contract: a caller that reads
/// `formatVersion`, `encoding` and `data` can reconstruct the octets without
/// guessing, and `data` is never interpreted as text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BytesValueWire {
    /// Wire format version, always [`BYTES_VALUE_FORMAT_VERSION`].
    pub format_version: String,
    /// One [`BytesEncoding`] spelling. An unknown spelling is rejected instead
    /// of guessed.
    pub encoding: String,
    /// Encoded octets.
    pub data: String,
}

/// Arbitrary octet sequence of the `bytes` scalar.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct BytesValue {
    octets: Vec<u8>,
}

impl BytesValue {
    /// Wraps octets exactly as the host received them.
    pub fn new(octets: Vec<u8>) -> Self {
        Self { octets }
    }

    /// Borrows the octets without decoding them.
    pub fn as_octets(&self) -> &[u8] {
        &self.octets
    }

    /// Returns the owned octets without decoding them.
    pub fn into_octets(self) -> Vec<u8> {
        self.octets
    }

    /// Number of octets, including NUL and non-UTF-8 octets.
    pub fn len(&self) -> usize {
        self.octets.len()
    }

    /// Whether this value is the empty octet sequence.
    pub fn is_empty(&self) -> bool {
        self.octets.is_empty()
    }

    /// Asks a storage implementation where it would keep this value.
    pub fn placement(&self, policy: &dyn BytesPlacementPolicy) -> BytesPlacement {
        policy.placement_for(&self.octets)
    }

    /// Builds the strict wire form of this value.
    pub fn to_wire(&self) -> BytesValueWire {
        BytesValueWire {
            format_version: BYTES_VALUE_FORMAT_VERSION.to_owned(),
            encoding: BytesEncoding::Hex.as_str().to_owned(),
            data: encode_hex(&self.octets),
        }
    }

    /// Reads the strict wire form back into octets.
    pub fn from_wire(wire: BytesValueWire) -> Result<Self, ArtifactError> {
        if wire.format_version != BYTES_VALUE_FORMAT_VERSION {
            return Err(ArtifactError {
                code: "BYT001".to_owned(),
                message: format!(
                    "unsupported bytes value format version {}",
                    wire.format_version
                ),
            });
        }
        let encoding = BytesEncoding::from_name(&wire.encoding).ok_or_else(|| ArtifactError {
            code: "BYT002".to_owned(),
            message: format!("unsupported bytes value encoding {}", wire.encoding),
        })?;
        match encoding {
            BytesEncoding::Hex => Ok(Self::new(decode_hex(&wire.data)?)),
        }
    }

    /// Serializes this value as its strict wire artifact.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.to_wire())
    }

    /// Reads and validates a strict bytes value artifact.
    pub fn from_json(input: &str) -> Result<Self, ArtifactError> {
        let wire: BytesValueWire = serde_json::from_str(input).map_err(|error| ArtifactError {
            code: "BYT005".to_owned(),
            message: error.to_string(),
        })?;
        Self::from_wire(wire)
    }
}

fn encode_hex(octets: &[u8]) -> String {
    let mut output = String::with_capacity(octets.len() * 2);
    for octet in octets {
        output.push(HEX_DIGITS[usize::from(octet >> 4)] as char);
        output.push(HEX_DIGITS[usize::from(octet & 0x0f)] as char);
    }
    output
}

fn decode_hex(data: &str) -> Result<Vec<u8>, ArtifactError> {
    let bytes = data.as_bytes();
    if !bytes.len().is_multiple_of(2) {
        return Err(ArtifactError {
            code: "BYT003".to_owned(),
            message: "bytes hex data must contain a whole number of octets".to_owned(),
        });
    }
    let mut octets = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.chunks(2) {
        octets.push(hex_digit(pair[0])? << 4 | hex_digit(pair[1])?);
    }
    Ok(octets)
}

fn hex_digit(byte: u8) -> Result<u8, ArtifactError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(ArtifactError {
            code: "BYT004".to_owned(),
            message: "bytes hex data must use lowercase hexadecimal digits".to_owned(),
        }),
    }
}
