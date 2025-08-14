#![allow(dead_code)]

#[derive(Debug, PartialEq, Eq)]
pub enum SerializationError {
    /// The module uses Wasm features not supported by serialization
    UnsupportedFeature { feature: &'static str },
    /// The module is in an invalid state for serialization
    InvalidModule { reason: &'static str },
    /// Internal serialization failure
    SerializationFailed { cause: &'static str },
    /// Module contains custom sections that cannot be serialized
    UnsupportedCustomSections,
    /// Module has lazy functions that cannot be serialized
    LazyFunctionsNotSupported,
}

impl core::fmt::Display for SerializationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SerializationError::UnsupportedFeature { feature } => {
                write!(f, "unsupported Wasm feature: {feature}")
            }
            SerializationError::InvalidModule { reason } => {
                write!(f, "invalid module state: {reason}")
            }
            SerializationError::SerializationFailed { cause } => {
                write!(f, "serialization failed: {cause}")
            }
            SerializationError::UnsupportedCustomSections => {
                write!(f, "module contains unsupported custom sections")
            }
            SerializationError::LazyFunctionsNotSupported => write!(
                f,
                "module contains lazy functions that cannot be serialized"
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SerializationError {}

#[derive(Debug, PartialEq, Eq)]
pub enum DeserializationError {
    /// The serialization format version is not supported
    UnsupportedVersion { version: u32, supported: u32 },
    /// The module requires Wasm features not supported by the engine
    FeatureMismatch { feature: &'static str },
    /// The serialized data is corrupted or malformed
    CorruptedData { reason: &'static str },
    /// Invalid indices or references in the serialized data
    InvalidIndex {
        index: u32,
        max: u32,
        context: &'static str,
    },
    /// Internal deserialization failure
    DeserializationFailed { cause: &'static str },
    /// The serialized data is incomplete or truncated
    IncompleteData { expected: usize, actual: usize },
    /// The serialized data is too large for the target system
    DataTooLarge { size: usize, max: usize },
}

#[allow(clippy::uninlined_format_args)]
impl core::fmt::Display for DeserializationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DeserializationError::UnsupportedVersion { version, supported } => write!(
                f,
                "unsupported serialization format version: {} (supported: {})",
                version, supported
            ),
            DeserializationError::FeatureMismatch { feature } => {
                write!(f, "required feature not supported by engine: {}", feature)
            }
            DeserializationError::CorruptedData { reason } => {
                write!(f, "corrupted or malformed serialized data: {}", reason)
            }
            DeserializationError::InvalidIndex {
                index,
                max,
                context,
            } => write!(
                f,
                "invalid index in serialized data: {} (max: {}) in {}",
                index, max, context
            ),
            DeserializationError::DeserializationFailed { cause } => {
                write!(f, "deserialization failed: {}", cause)
            }
            DeserializationError::IncompleteData { expected, actual } => write!(
                f,
                "incomplete serialized data: expected {} bytes, got {}",
                expected, actual
            ),
            DeserializationError::DataTooLarge { size, max } => write!(
                f,
                "serialized data too large: {} bytes (max: {})",
                size, max
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DeserializationError {}

impl From<crate::Error> for DeserializationError {
    fn from(_err: crate::Error) -> Self {
        DeserializationError::DeserializationFailed {
            cause: "module construction error",
        }
    }
}
