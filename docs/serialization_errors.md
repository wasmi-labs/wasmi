# Serialization and Deserialization Error Types

## Overview

This document defines the error types used for Wasmi IR serialization and deserialization operations. All errors are designed to be informative and actionable, and are implemented as plain enums with manual trait implementations for maximum portability (including no-std targets).

## Error Types

### SerializationError

```rust
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
            SerializationError::UnsupportedFeature { feature } => write!(f, "unsupported Wasm feature: {}", feature),
            SerializationError::InvalidModule { reason } => write!(f, "invalid module state: {}", reason),
            SerializationError::SerializationFailed { cause } => write!(f, "serialization failed: {}", cause),
            SerializationError::UnsupportedCustomSections => write!(f, "module contains unsupported custom sections"),
            SerializationError::LazyFunctionsNotSupported => write!(f, "module contains lazy functions that cannot be serialized"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SerializationError {}
```

### DeserializationError

```rust
#[derive(Debug, PartialEq, Eq)]
pub enum DeserializationError {
    /// The serialization format version is not supported
    UnsupportedVersion { version: u32, supported: u32 },
    /// The module requires Wasm features not supported by the engine
    FeatureMismatch { feature: &'static str },
    /// The serialized data is corrupted or malformed
    CorruptedData { reason: &'static str },
    /// Invalid indices or references in the serialized data
    InvalidIndex { index: u32, max: u32, context: &'static str },
    /// Internal deserialization failure
    DeserializationFailed { cause: &'static str },
    /// The serialized data is incomplete or truncated
    IncompleteData { expected: usize, actual: usize },
    /// The serialized data is too large for the target system
    DataTooLarge { size: usize, max: usize },
}

impl core::fmt::Display for DeserializationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DeserializationError::UnsupportedVersion { version, supported } => write!(f, "unsupported serialization format version: {} (supported: {})", version, supported),
            DeserializationError::FeatureMismatch { feature } => write!(f, "required feature not supported by engine: {}", feature),
            DeserializationError::CorruptedData { reason } => write!(f, "corrupted or malformed serialized data: {}", reason),
            DeserializationError::InvalidIndex { index, max, context } => write!(f, "invalid index in serialized data: {} (max: {}) in {}", index, max, context),
            DeserializationError::DeserializationFailed { cause } => write!(f, "deserialization failed: {}", cause),
            DeserializationError::IncompleteData { expected, actual } => write!(f, "incomplete serialized data: expected {} bytes, got {}", expected, actual),
            DeserializationError::DataTooLarge { size, max } => write!(f, "serialized data too large: {} bytes (max: {})", size, max),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DeserializationError {}
```

## Error Context

### SerializationError Context

#### UnsupportedFeature
- **When:** Module uses Wasm proposals not yet supported by serialization
- **Examples:** 
  - `exception-handling` proposal
  - Future proposals not yet implemented
- **Action:** Update serialization format or reject module

#### InvalidModule
- **When:** Module is in an inconsistent or invalid state
- **Examples:**
  - Uncompiled functions in eager compilation mode
  - Missing function bodies
  - Invalid references
- **Action:** Fix module state before serialization

#### SerializationFailed
- **When:** Internal serialization process fails
- **Examples:**
  - Memory allocation failure
  - Postcard serialization error
  - Unexpected data structure
- **Action:** Check system resources and module integrity

#### UnsupportedCustomSections
- **When:** Module contains custom sections that cannot be serialized
- **Examples:**
  - Debug sections
  - Source maps
  - Vendor-specific sections
- **Action:** Strip custom sections before serialization

#### LazyFunctionsNotSupported
- **When:** Module contains functions that haven't been compiled yet
- **Examples:**
  - Lazy compilation mode with uncompiled functions
- **Action:** Compile all functions before serialization

### DeserializationError Context

#### UnsupportedVersion
- **When:** Serialized data uses a format version not supported by the deserializer
- **Examples:**
  - Version 2 data with version 1 deserializer
  - Future format versions
- **Action:** Update deserializer or use compatible version

#### FeatureMismatch
- **When:** Module requires features not supported by the target engine
- **Examples:**
  - SIMD features on non-SIMD engine
  - Memory64 on 32-bit engine
  - New proposals on old engine
- **Action:** Use compatible engine or update engine features

#### CorruptedData
- **When:** Serialized data is corrupted or malformed
- **Examples:**
  - Truncated data
  - Invalid enum values
  - Malformed postcard data
- **Action:** Re-serialize the module or check data integrity

#### InvalidIndex
- **When:** Serialized data contains invalid indices or references
- **Examples:**
  - Function type index out of bounds
  - Import index exceeds import count
  - Invalid export reference
- **Action:** Check serialization process or re-serialize

#### DeserializationFailed
- **When:** Internal deserialization process fails
- **Examples:**
  - Memory allocation failure
  - Postcard deserialization error
  - Unexpected data structure
- **Action:** Check system resources and data integrity

#### IncompleteData
- **When:** Serialized data is truncated or incomplete
- **Examples:**
  - Network transmission error
  - File corruption
  - Buffer overflow
- **Action:** Re-transmit or re-serialize the module

#### DataTooLarge
- **When:** Serialized data exceeds system limits
- **Examples:**
  - Module too large for embedded target
  - Memory constraints
  - System limits
- **Action:** Use smaller module or increase system limits

## Error Handling Guidelines

### For Serialization
1. **Validate early:** Check module state before starting serialization
2. **Be specific:** Provide detailed error messages with context
3. **Fail fast:** Stop at first error to avoid partial results
4. **Include context:** Add relevant module information to errors

### For Deserialization
1. **Validate format:** Check version and basic structure first
2. **Check features:** Validate required features against engine capabilities
3. **Validate indices:** Ensure all references are within bounds
4. **Fail safely:** Never proceed with invalid data
5. **Provide context:** Include relevant information for debugging

### Error Recovery
- **Serialization errors:** Usually require fixing the module or system
- **Deserialization errors:** Usually require re-serializing or using compatible data
- **Version mismatches:** Require format updates or compatibility layers
- **Feature mismatches:** Require engine updates or feature negotiation

## Integration with Wasmi Error System

### Error Conversion
```rust
impl From<SerializationError> for wasmi::Error {
    fn from(err: SerializationError) -> Self {
        wasmi::Error::new(format!("serialization error: {}", err))
    }
}

impl From<DeserializationError> for wasmi::Error {
    fn from(err: DeserializationError) -> Self {
        wasmi::Error::new(format!("deserialization error: {}", err))
    }
}
```

## Testing Error Conditions

### Serialization Error Tests
- Modules with unsupported features
- Invalid module states
- Memory pressure conditions
- Custom sections
- Lazy functions

### Deserialization Error Tests
- Corrupted data
- Version mismatches
- Feature mismatches
- Invalid indices
- Truncated data
- Oversized data

### Error Message Tests
- Verify error messages are informative
- Check that context is included
- Ensure actionable guidance is provided 