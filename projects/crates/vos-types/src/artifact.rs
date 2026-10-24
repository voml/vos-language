use serde::{Deserialize, Serialize};

use crate::ArtifactError;
use crate::contract::{
    CONTRACT_FINGERPRINT_VERSION, OperationContract, ServiceContract,
    operation_contract_fingerprint, service_contract_fingerprint,
};
use crate::ids::canonical_digest;

/// Product contract format that consumes this artifact.
///
/// The wire form carries the complete `/v0` format string, so a host cannot
/// accept a contract for a different product or a different contract version by
/// accident. Any other string is rejected while reading.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContractFormat {
    /// Contract consumed by Iris.
    #[serde(rename = "vos/iris.contract/v0")]
    Iris,
    /// Contract consumed by Hermes.
    #[serde(rename = "vos/hermes.contract/v0")]
    Hermes,
}

impl ContractFormat {
    /// Contract version namespace shared by every product format.
    pub const VERSION_TAG: &'static str = "v0";

    /// Every supported product format.
    pub const SUPPORTED: [Self; 2] = [Self::Iris, Self::Hermes];

    /// Returns the complete wire form of this format.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Iris => "vos/iris.contract/v0",
            Self::Hermes => "vos/hermes.contract/v0",
        }
    }
}

impl core::fmt::Display for ContractFormat {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Minimal producer identity carried by a contract artifact.
///
/// The producer is provenance only: it never contributes to the contract
/// fingerprint, so upgrading the producer does not change the contract.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContractProducer {
    /// Producer name.
    pub name: String,
    /// Producer version.
    pub version: String,
}

impl ContractProducer {
    /// Builds a producer identity.
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_owned(),
            version: version.to_owned(),
        }
    }

    /// Producer identity of this crate.
    pub fn vos() -> Self {
        Self::new("vos-language", env!("CARGO_PKG_VERSION"))
    }
}

/// Operations and services carried by a contract artifact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContractPayload {
    /// Validated operation contracts.
    pub operations: Vec<OperationContract>,
    /// Validated service contracts.
    pub services: Vec<ServiceContract>,
}

/// Versioned contract artifact consumed by Iris and Hermes.
///
/// The artifact is the only contract surface a downstream host reads. It
/// carries the product format, the producer, one canonical fingerprint and the
/// validated payload, and it rejects unknown formats, unknown fields and stale
/// fingerprints while reading.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContractArtifact {
    /// Product contract format including its contract version.
    pub version: ContractFormat,
    /// Producer provenance.
    pub producer: ContractProducer,
    /// Canonical fingerprint of the payload.
    pub fingerprint: String,
    /// Validated operations and services.
    pub payload: ContractPayload,
}

impl ContractArtifact {
    /// Seals a payload into an artifact by computing its canonical fingerprint.
    pub fn seal(
        version: ContractFormat,
        producer: ContractProducer,
        payload: ContractPayload,
    ) -> Self {
        let fingerprint = contract_payload_fingerprint(&payload);
        Self {
            version,
            producer,
            fingerprint,
            payload,
        }
    }

    /// Rejects an artifact a downstream host cannot trust.
    ///
    /// Errors from the payload keep their operation or service diagnostic code
    /// so a host can report the exact offending contract.
    pub fn validate(&self) -> Result<(), ArtifactError> {
        if self.producer.name.trim().is_empty() || self.producer.version.trim().is_empty() {
            return Err(ArtifactError {
                code: "ART021".to_owned(),
                message: "contract artifact producer must carry a name and a version".to_owned(),
            });
        }
        if self.fingerprint != contract_payload_fingerprint(&self.payload) {
            return Err(ArtifactError {
                code: "ART020".to_owned(),
                message: "contract artifact fingerprint does not match its payload".to_owned(),
            });
        }
        for operation in &self.payload.operations {
            operation.validate()?;
        }
        for service in &self.payload.services {
            service.validate()?;
        }
        Ok(())
    }

    /// Serializes this artifact with stable field names.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Reads and validates a strict contract artifact.
    pub fn from_json(input: &str) -> Result<Self, ArtifactError> {
        let artifact: Self = serde_json::from_str(input).map_err(|error| ArtifactError {
            code: "ART010".to_owned(),
            message: error.to_string(),
        })?;
        artifact.validate()?;
        Ok(artifact)
    }
}

/// Canonical fingerprint of a contract payload.
///
/// The digest binds the operation and service fingerprints, so a host compares
/// one value while still detecting any change to either contract family.
pub fn contract_payload_fingerprint(payload: &ContractPayload) -> String {
    canonical_digest(&FingerprintInput {
        fingerprint_version: CONTRACT_FINGERPRINT_VERSION,
        operations: operation_contract_fingerprint(&payload.operations),
        services: service_contract_fingerprint(&payload.services),
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FingerprintInput {
    fingerprint_version: &'static str,
    operations: String,
    services: String,
}
