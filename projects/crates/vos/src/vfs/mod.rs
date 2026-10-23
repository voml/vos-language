//! Virtual file system abstraction for VOS multi-file resolution.
//!
//! The [`Vfs`] trait decouples path resolution and file reading from the
//! concrete host. A default [`FsVfs`] implementation reads from the local
//! file system; WASM / in-memory hosts can implement the trait themselves.

use std::fmt;
use std::path::{Path, PathBuf};

/// Structured error kind produced by a [`Vfs`] implementation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VfsErrorKind {
    /// The referenced file does not exist.
    NotFound,
    /// A `using` chain loops back to a previously loaded source.
    Circular,
    /// The import path is malformed or escapes its root.
    InvalidPath,
    /// An I/O failure surfaced while reading the file.
    Io,
}

impl fmt::Display for VfsErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => f.write_str("not found"),
            Self::Circular => f.write_str("circular import"),
            Self::InvalidPath => f.write_str("invalid path"),
            Self::Io => f.write_str("i/o error"),
        }
    }
}

/// A structured VFS failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VfsError {
    /// Stable error category.
    pub kind: VfsErrorKind,
    /// Additional host-provided context.
    pub message: String,
}

impl VfsError {
    /// Builds an error with the given kind and message.
    pub fn new<S: Into<String>>(kind: VfsErrorKind, message: S) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    /// Convenience constructor for [`VfsErrorKind::NotFound`].
    pub fn not_found<P: fmt::Display>(path: P) -> Self {
        Self::new(VfsErrorKind::NotFound, format!("file not found: {path}"))
    }

    /// Convenience constructor for [`VfsErrorKind::Circular`].
    pub fn circular<P: fmt::Display>(path: P) -> Self {
        Self::new(
            VfsErrorKind::Circular,
            format!("circular using detected: {path}"),
        )
    }

    /// Convenience constructor for [`VfsErrorKind::InvalidPath`].
    pub fn invalid<P: fmt::Display>(path: P) -> Self {
        Self::new(
            VfsErrorKind::InvalidPath,
            format!("invalid import path: {path}"),
        )
    }
}

impl fmt::Display for VfsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl std::error::Error for VfsError {}

/// Virtual file system contract used by the VOS multi-file resolver.
pub trait Vfs {
    /// Reads the entire source text at `path`.
    fn read_file(&self, path: &Path) -> Result<String, VfsError>;

    /// Resolves a possibly-relative import string against `base`.
    ///
    /// The returned path should be absolute so that subsequent lookups and
    /// cycle detection have a stable key.
    fn resolve(&self, base: &Path, import: &str) -> Result<PathBuf, VfsError>;
}

/// Default [`Vfs`] implementation backed by the local file system.
#[derive(Clone, Debug, Default)]
pub struct FsVfs;

impl FsVfs {
    /// Creates a new `FsVfs`.
    pub fn new() -> Self {
        Self
    }
}

impl Vfs for FsVfs {
    fn read_file(&self, path: &Path) -> Result<String, VfsError> {
        std::fs::read_to_string(path).map_err(|err| match err.kind() {
            std::io::ErrorKind::NotFound => VfsError::not_found(path.display()),
            _ => VfsError::new(
                VfsErrorKind::Io,
                format!("failed to read {}: {err}", path.display()),
            ),
        })
    }

    fn resolve(&self, base: &Path, import: &str) -> Result<PathBuf, VfsError> {
        let import_path = Path::new(import);
        if import.is_empty() {
            return Err(VfsError::invalid(import));
        }
        let base_dir = if base.is_dir() {
            base.to_path_buf()
        } else {
            base.parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("."))
        };
        let joined = base_dir.join(import_path);
        let canonical = joined
            .canonicalize()
            .map_err(|_| VfsError::not_found(joined.display()))?;
        Ok(canonical)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn fs_vfs_reads_existing_file() {
        let dir = tempfile_dir();
        let path = dir.join("a.vos");
        fs::write(&path, "table User { id: uuid, }").unwrap();
        let vfs = FsVfs::new();
        assert_eq!(vfs.read_file(&path).unwrap(), "table User { id: uuid, }");
    }

    #[test]
    fn fs_vfs_returns_not_found_for_missing_file() {
        let vfs = FsVfs::new();
        let err = vfs
            .read_file(Path::new("/definitely/not/here.vos"))
            .unwrap_err();
        assert_eq!(err.kind, VfsErrorKind::NotFound);
    }

    #[test]
    fn fs_vfs_resolves_relative_import() {
        let dir = tempfile_dir();
        let base = dir.join("main.vos");
        let target = dir.join("shared.vos");
        fs::write(&target, "").unwrap();
        let vfs = FsVfs::new();
        let resolved = vfs.resolve(&base, "shared.vos").unwrap();
        assert_eq!(resolved.file_name().unwrap(), "shared.vos");
    }

    #[test]
    fn fs_vfs_rejects_empty_import() {
        let vfs = FsVfs::new();
        let err = vfs.resolve(Path::new("."), "").unwrap_err();
        assert_eq!(err.kind, VfsErrorKind::InvalidPath);
    }

    fn tempfile_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "vos-vfs-tests-{}-{}",
            std::process::id(),
            rand_suffix()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn rand_suffix() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_string()
    }
}
