//! Canonical model of the fixed-dimension `vector<N>` logical type.
//!
//! A `vector<N>` is a `N`-component floating-point embedding whose precision and
//! dimension are part of the contract. VOS closes the value contract only: the
//! element precision, the positive dimension, the rejection of non-finite
//! components and the component order. Distance metrics, indexes and approximate
//! nearest-neighbour search are query capabilities and stay out of this type.

use serde::{Deserialize, Serialize};

use crate::ArtifactError;

/// Version of the strict `vector<N>` value wire shape.
pub const VECTOR_VALUE_FORMAT_VERSION: &str = "vos-vector-value-v0";

/// Element precision of a `vector<N>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VectorElement {
    /// 32-bit IEEE-754 component, the precision a `vector<N>` resolves to.
    F32,
}

impl VectorElement {
    /// Every element precision accepted by the contract.
    pub const ALL: [Self; 1] = [Self::F32];

    /// Returns the canonical wire spelling of this precision.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::F32 => "f32",
        }
    }

    /// Reads a precision by its exact wire spelling.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|element| element.as_str() == name)
    }
}

/// Strict wire form of a `vector<N>` value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VectorValueWire {
    /// Wire format version, always [`VECTOR_VALUE_FORMAT_VERSION`].
    pub format_version: String,
    /// One [`VectorElement`] spelling. An unknown spelling is rejected.
    pub element: String,
    /// Dimension fixed by the contract.
    pub dimension: u32,
    /// Components in ascending index order.
    pub components: Vec<f32>,
}

/// Fixed-dimension vector of finite `f32` components.
#[derive(Clone, Debug, PartialEq)]
pub struct VectorValue {
    dimension: u32,
    components: Vec<f32>,
}

impl VectorValue {
    /// Builds a value and rejects a zero dimension, a wrong component count or a
    /// non-finite component.
    pub fn new(dimension: u32, components: Vec<f32>) -> Result<Self, ArtifactError> {
        let value = Self {
            dimension,
            components,
        };
        value.validate()?;
        Ok(value)
    }

    /// Dimension fixed by the contract.
    pub fn dimension(&self) -> u32 {
        self.dimension
    }

    /// Components in ascending index order.
    pub fn components(&self) -> &[f32] {
        &self.components
    }

    /// Element precision of this value.
    pub fn element(&self) -> VectorElement {
        VectorElement::F32
    }

    /// Validates the dimension, the component count and component finiteness.
    pub fn validate(&self) -> Result<(), ArtifactError> {
        if self.dimension == 0 {
            return Err(ArtifactError {
                code: "VEC001".to_owned(),
                message: "vector dimension must be positive".to_owned(),
            });
        }
        if self.components.len() != self.dimension as usize {
            return Err(ArtifactError {
                code: "VEC002".to_owned(),
                message: format!(
                    "vector of dimension {} must carry {} components",
                    self.dimension, self.dimension
                ),
            });
        }
        if self
            .components
            .iter()
            .any(|component| !component.is_finite())
        {
            return Err(ArtifactError {
                code: "VEC003".to_owned(),
                message: "vector components must be finite and must not be NaN or infinity"
                    .to_owned(),
            });
        }
        Ok(())
    }

    /// Builds the strict wire form. Components are written in index order and are
    /// never reordered or normalized.
    pub fn to_wire(&self) -> VectorValueWire {
        VectorValueWire {
            format_version: VECTOR_VALUE_FORMAT_VERSION.to_owned(),
            element: VectorElement::F32.as_str().to_owned(),
            dimension: self.dimension,
            components: self.components.clone(),
        }
    }

    /// Reads the strict wire form and revalidates every constraint.
    pub fn from_wire(wire: VectorValueWire) -> Result<Self, ArtifactError> {
        if wire.format_version != VECTOR_VALUE_FORMAT_VERSION {
            return Err(ArtifactError {
                code: "VEC005".to_owned(),
                message: format!(
                    "unsupported vector value format version {}",
                    wire.format_version
                ),
            });
        }
        if VectorElement::from_name(&wire.element).is_none() {
            return Err(ArtifactError {
                code: "VEC004".to_owned(),
                message: format!("unsupported vector element precision {}", wire.element),
            });
        }
        Self::new(wire.dimension, wire.components)
    }

    /// Serializes this value as its strict wire artifact.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.to_wire())
    }

    /// Reads and validates a strict vector value artifact.
    pub fn from_json(input: &str) -> Result<Self, ArtifactError> {
        let wire: VectorValueWire = serde_json::from_str(input).map_err(|error| ArtifactError {
            code: "VEC006".to_owned(),
            message: error.to_string(),
        })?;
        Self::from_wire(wire)
    }
}
