//! Canonical descriptor of the `file` logical type.
//!
//! A `file` is a large logical object rather than a scalar. VOS freezes only its
//! description: the **manifest identity** (content digest and octet length), the
//! **presentation metadata** (name and media type), and the observable read and
//! lifecycle semantics. Chunk layout, object-store keys, caching, retries and
//! physical garbage collection remain storage implementation details, so no
//! VOS value depends on a specific storage API.
//!
//! Identity and metadata are deliberately separate: renaming a file or relabelling
//! its media type never changes the content hash, and two descriptors with the
//! same octets and different names share one [`FileManifestIdentity`].

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::ArtifactError;

/// Version of the strict `file` descriptor wire shape.
pub const FILE_DESCRIPTOR_FORMAT_VERSION: &str = "vos-file-descriptor-v0";

/// Presentation metadata of a file, never part of its identity.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FileMetadata {
    /// Original file name, absent for an anonymous content-addressed object.
    pub name: Option<String>,
    /// Declared media type, absent when the producer declared none.
    pub media_type: Option<String>,
}

/// Content identity of a file as one manifest object.
///
/// The digest covers the complete octets and the length counts every octet, so a
/// storage implementation may split the file into any number of chunks without
/// changing this identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FileManifestIdentity {
    /// Lowercase hexadecimal SHA-256 over the complete octets.
    pub digest: String,
    /// Total octet count of the complete file.
    pub octets: u64,
}

impl FileManifestIdentity {
    /// Computes the manifest identity of a complete octet sequence.
    pub fn from_octets(octets: &[u8]) -> Self {
        let digest = Sha256::digest(octets);
        Self {
            digest: digest
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>(),
            octets: octets.len() as u64,
        }
    }

    /// Validates the digest spelling and the octet count.
    pub fn validate(&self) -> Result<(), ArtifactError> {
        let shaped = self.digest.len() == 64
            && self
                .digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
        if !shaped {
            return Err(ArtifactError {
                code: "FIL002".to_owned(),
                message: "file identity digest must be 64 lowercase hexadecimal digits".to_owned(),
            });
        }
        Ok(())
    }
}

/// A read request expressed in octets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FileReadWindow {
    /// First octet of the window.
    pub offset: u64,
    /// Requested octet count. A resolved window may be shorter than this.
    pub length: u64,
}

impl FileReadWindow {
    /// Builds a read request or a resolved window.
    pub fn new(offset: u64, length: u64) -> Self {
        Self { offset, length }
    }

    /// One octet past the last octet addressed by this window.
    pub fn end(&self) -> u64 {
        self.offset.saturating_add(self.length)
    }
}

/// Observable lifecycle state of one descriptor reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FileReferenceState {
    /// The reference is usable and pins the content.
    Live,
    /// The reference was released and no longer pins the content.
    Released,
}

/// Strict descriptor of a `file` value.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FileDescriptor {
    /// Descriptor format version, always [`FILE_DESCRIPTOR_FORMAT_VERSION`].
    pub format_version: String,
    /// Content identity, independent of metadata.
    pub identity: FileManifestIdentity,
    /// Presentation metadata, independent of identity.
    pub metadata: FileMetadata,
}

impl FileDescriptor {
    /// Builds a descriptor from a known identity and its metadata.
    pub fn new(identity: FileManifestIdentity, metadata: FileMetadata) -> Self {
        Self {
            format_version: FILE_DESCRIPTOR_FORMAT_VERSION.to_owned(),
            identity,
            metadata,
        }
    }

    /// Describes a complete octet sequence, which is the observable creation step.
    pub fn describe(octets: &[u8], metadata: FileMetadata) -> Self {
        Self::new(FileManifestIdentity::from_octets(octets), metadata)
    }

    /// Total octet count of the file.
    pub fn octets(&self) -> u64 {
        self.identity.octets
    }

    /// The window that addresses every octet of the file.
    pub fn full_window(&self) -> FileReadWindow {
        FileReadWindow::new(0, self.identity.octets)
    }

    /// Whether an offset starts at or beyond end of file.
    pub fn is_eof(&self, offset: u64) -> bool {
        offset >= self.identity.octets
    }

    /// Resolves a read request against the file length.
    ///
    /// A request that starts before end of file is clamped to the remaining
    /// octets, so a range read past end of file is observable as a short read. A
    /// request that starts at or beyond end of file yields an empty window at
    /// that offset instead of an error.
    pub fn window(&self, request: FileReadWindow) -> FileReadWindow {
        let total = self.identity.octets;
        if request.offset >= total {
            return FileReadWindow::new(request.offset, 0);
        }
        FileReadWindow::new(request.offset, request.length.min(total - request.offset))
    }

    /// Validates version, content identity and metadata.
    pub fn validate(&self) -> Result<(), ArtifactError> {
        if self.format_version != FILE_DESCRIPTOR_FORMAT_VERSION {
            return Err(ArtifactError {
                code: "FIL001".to_owned(),
                message: format!(
                    "unsupported file descriptor format version {}",
                    self.format_version
                ),
            });
        }
        self.identity.validate()?;
        for label in [
            self.metadata.name.as_deref(),
            self.metadata.media_type.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            if label.trim().is_empty() {
                return Err(ArtifactError {
                    code: "FIL003".to_owned(),
                    message: "declared file metadata must not be empty".to_owned(),
                });
            }
        }
        Ok(())
    }

    /// Serializes this descriptor as its strict wire artifact.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Reads and validates a strict file descriptor artifact.
    pub fn from_json(input: &str) -> Result<Self, ArtifactError> {
        let descriptor: Self = serde_json::from_str(input).map_err(|error| ArtifactError {
            code: "FIL006".to_owned(),
            message: error.to_string(),
        })?;
        descriptor.validate()?;
        Ok(descriptor)
    }
}

/// One live or released reference to a descriptor.
///
/// A storage implementation may reclaim the octets only when no live reference
/// remains, and it reports released references through [`FileReference::load`]
/// instead of silently returning empty content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileReference {
    descriptor: FileDescriptor,
    state: FileReferenceState,
}

impl FileReference {
    /// Opens a live reference to a descriptor.
    pub fn live(descriptor: FileDescriptor) -> Self {
        Self {
            descriptor,
            state: FileReferenceState::Live,
        }
    }

    /// Current lifecycle state of this reference.
    pub fn state(&self) -> FileReferenceState {
        self.state
    }

    /// Borrows the descriptor this reference pins.
    pub fn descriptor(&self) -> &FileDescriptor {
        &self.descriptor
    }

    /// Resolves a read against the descriptor while this reference is live.
    pub fn load(&self, request: FileReadWindow) -> Result<FileReadWindow, ArtifactError> {
        if self.state == FileReferenceState::Released {
            return Err(ArtifactError {
                code: "FIL005".to_owned(),
                message: "file reference was released before the read".to_owned(),
            });
        }
        Ok(self.descriptor.window(request))
    }

    /// Releases this reference. Releasing an already released reference is a no-op.
    pub fn release(&mut self) {
        self.state = FileReferenceState::Released;
    }
}
