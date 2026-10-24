use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{ArtifactError, OperationKind};

/// Version of the operation, service and method identity derivation rules.
pub const CONTRACT_IDENTITY_VERSION: &str = "vos-contract-identity-v0";
/// Prefix of every durable operation identity.
pub const OPERATION_ID_PREFIX: &str = "op_";
/// Prefix of every durable service identity.
pub const SERVICE_ID_PREFIX: &str = "svc_";
/// Prefix of every durable service method identity.
pub const METHOD_ID_PREFIX: &str = "mtd_";
/// Number of lowercase hexadecimal characters carried by every identity.
pub const ID_HEX_LEN: usize = 64;

/// Canonical, source-independent operation key that a durable
/// [`OperationId`] is derived from.
///
/// The key carries no source text, span, comment or file identity, so the
/// identity of an operation changes only when its declaration kind, name or
/// signature changes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OperationIdentityKey {
    /// Identity derivation version.
    pub identity_version: String,
    /// Canonical namespace-qualified path. The last segment is the operation
    /// name, so two identically named operations in different namespaces keep
    /// distinct identities.
    pub canonical_path: Vec<String>,
    /// Declaration kind.
    pub kind: OperationKind,
    /// Canonical operation name.
    pub name: String,
    /// Named parameter tuple in declaration order. Parameter order is semantic.
    pub parameters: Vec<OperationParameterKey>,
    /// Canonical result type key, `unit` when the declaration has no result.
    pub result: String,
}

impl OperationIdentityKey {
    /// Builds a validated identity key.
    ///
    /// Empty names, empty namespace segments, empty type keys and duplicate
    /// parameter names are rejected instead of being canonicalized silently.
    pub fn new(
        namespace: &[String],
        kind: OperationKind,
        name: &str,
        parameters: Vec<OperationParameterKey>,
        result: &str,
    ) -> Result<Self, ArtifactError> {
        validate_name(name, "ID033", "operation name")?;
        if namespace.iter().any(|segment| segment.trim().is_empty()) {
            return Err(ArtifactError {
                code: "ID038".to_owned(),
                message: "operation namespace segments must not be empty".to_owned(),
            });
        }
        validate_key_text(result, "operation result type")?;
        let mut names = std::collections::BTreeSet::new();
        for parameter in &parameters {
            if !names.insert(parameter.name.as_str()) {
                return Err(ArtifactError {
                    code: "ID034".to_owned(),
                    message: "operation parameter names must be unique".to_owned(),
                });
            }
        }
        let mut canonical_path = namespace.to_vec();
        canonical_path.push(name.to_owned());
        Ok(Self {
            identity_version: CONTRACT_IDENTITY_VERSION.to_owned(),
            canonical_path,
            kind,
            name: name.to_owned(),
            parameters,
            result: result.to_owned(),
        })
    }
}

/// One named parameter in an operation identity key.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OperationParameterKey {
    /// Parameter name as part of the operation signature.
    pub name: String,
    /// Canonical parameter type key.
    pub type_key: String,
}

impl OperationParameterKey {
    /// Builds a validated parameter key.
    pub fn new(name: &str, type_key: &str) -> Result<Self, ArtifactError> {
        validate_name(name, "ID033", "operation parameter name")?;
        validate_key_text(type_key, "operation parameter type")?;
        Ok(Self {
            name: name.to_owned(),
            type_key: type_key.to_owned(),
        })
    }
}

/// Canonical, source-independent service key that a durable [`ServiceId`] is
/// derived from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ServiceIdentityKey {
    /// Identity derivation version.
    pub identity_version: String,
    /// Canonical namespace-qualified path, including the service name.
    pub canonical_path: Vec<String>,
}

impl ServiceIdentityKey {
    /// Builds a validated service key.
    pub fn new(canonical_path: Vec<String>) -> Result<Self, ArtifactError> {
        if canonical_path.is_empty() || canonical_path.iter().any(String::is_empty) {
            return Err(ArtifactError {
                code: "ID035".to_owned(),
                message: "service canonical path must have non-empty segments".to_owned(),
            });
        }
        Ok(Self {
            identity_version: CONTRACT_IDENTITY_VERSION.to_owned(),
            canonical_path,
        })
    }
}

/// Canonical, source-independent method key that a durable [`MethodId`] is
/// derived from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MethodIdentityKey {
    /// Identity derivation version.
    pub identity_version: String,
    /// Durable identity of the owning service.
    pub service_id: ServiceId,
    /// Canonical method name.
    pub name: String,
}

impl MethodIdentityKey {
    /// Builds a validated method key.
    pub fn new(service_id: ServiceId, name: &str) -> Result<Self, ArtifactError> {
        validate_name(name, "ID036", "service method name")?;
        Ok(Self {
            identity_version: CONTRACT_IDENTITY_VERSION.to_owned(),
            service_id,
            name: name.to_owned(),
        })
    }
}

/// Durable identity of a VOS operation.
///
/// The wire form is `op_` followed by 64 lowercase hexadecimal characters and
/// is derived from [`OperationIdentityKey`].
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OperationId(String);

impl OperationId {
    /// Builds the identity of an operation key.
    pub fn derive(key: &OperationIdentityKey) -> Self {
        Self(derive_identity(OPERATION_ID_PREFIX, key))
    }

    /// Reads a durable operation identity and rejects malformed input.
    pub fn parse(value: &str) -> Result<Self, ArtifactError> {
        validate_identity(value, OPERATION_ID_PREFIX, "ID030", "operation id")?;
        Ok(Self(value.to_owned()))
    }

    /// Returns the wire form of this identity.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl core::fmt::Display for OperationId {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Durable identity of a VOS service.
///
/// The wire form is `svc_` followed by 64 lowercase hexadecimal characters and
/// is derived from [`ServiceIdentityKey`].
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ServiceId(String);

impl ServiceId {
    /// Builds the identity of a service key.
    pub fn derive(key: &ServiceIdentityKey) -> Self {
        Self(derive_identity(SERVICE_ID_PREFIX, key))
    }

    /// Reads a durable service identity and rejects malformed input.
    pub fn parse(value: &str) -> Result<Self, ArtifactError> {
        validate_identity(value, SERVICE_ID_PREFIX, "ID031", "service id")?;
        Ok(Self(value.to_owned()))
    }

    /// Returns the wire form of this identity.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl core::fmt::Display for ServiceId {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Durable identity of one service method.
///
/// The wire form is `mtd_` followed by 64 lowercase hexadecimal characters and
/// is derived from [`MethodIdentityKey`]. A method identity depends only on the
/// owning service identity and the method name, so transport bindings cannot
/// change it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MethodId(String);

impl MethodId {
    /// Builds the identity of a service method key.
    pub fn derive(key: &MethodIdentityKey) -> Self {
        Self(derive_identity(METHOD_ID_PREFIX, key))
    }

    /// Reads a durable method identity and rejects malformed input.
    pub fn parse(value: &str) -> Result<Self, ArtifactError> {
        validate_identity(value, METHOD_ID_PREFIX, "ID032", "method id")?;
        Ok(Self(value.to_owned()))
    }

    /// Returns the wire form of this identity.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl core::fmt::Display for MethodId {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(&self.0)
    }
}

fn derive_identity<T: Serialize>(prefix: &str, key: &T) -> String {
    let wire = serde_json::to_vec(key).expect("contract identity keys serialize");
    let mut identity = String::with_capacity(prefix.len() + ID_HEX_LEN);
    identity.push_str(prefix);
    for byte in Sha256::digest(wire) {
        identity.push_str(&format!("{byte:02x}"));
    }
    identity
}

fn validate_identity(
    value: &str,
    prefix: &str,
    code: &str,
    label: &str,
) -> Result<(), ArtifactError> {
    let malformed = ArtifactError {
        code: code.to_owned(),
        message: format!("{label} must be `{prefix}` plus {ID_HEX_LEN} lowercase hex characters"),
    };
    let Some(hex) = value.strip_prefix(prefix) else {
        return Err(malformed);
    };
    if hex.len() != ID_HEX_LEN
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(malformed);
    }
    Ok(())
}

fn validate_name(name: &str, code: &str, label: &str) -> Result<(), ArtifactError> {
    if name.trim().is_empty() || name.chars().any(char::is_whitespace) {
        return Err(ArtifactError {
            code: code.to_owned(),
            message: format!("{label} must be a non-empty name without whitespace"),
        });
    }
    Ok(())
}

fn validate_key_text(value: &str, label: &str) -> Result<(), ArtifactError> {
    if value.trim().is_empty() || value.chars().any(char::is_whitespace) {
        return Err(ArtifactError {
            code: "ID037".to_owned(),
            message: format!("{label} must be a non-empty canonical key"),
        });
    }
    Ok(())
}
