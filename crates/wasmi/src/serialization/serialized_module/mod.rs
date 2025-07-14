//! Serialized module data structures.
//!
//! This module defines the data structures used to represent a Wasmi module
//! in its serialized form, optimized for compactness and cross-platform compatibility.

extern crate alloc;
use crate::serialization::serialized_module::types::{
    SerializedExport, SerializedFeatures, SerializedFuncType, SerializedGlobal, SerializedImport,
    SerializedInternalFunc,
};
use alloc::vec::Vec;

pub(crate) mod types;

pub(super) use types::{
    SerializedDataSegment, SerializedElementSegment, SerializedExternType, SerializedMemoryType,
    SerializedTableType,
};

/// Current serialization format version.
pub const SERIALIZATION_VERSION: u32 = 1;

/// Serialized representation of a Wasmi module.
///
/// This struct contains all the essential data needed to reconstruct a Wasmi module
/// for execution, excluding runtime-specific data, custom sections, and debug information.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialization", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialization", derive(serde::Deserialize))]
pub struct SerializedModule {
    /// Format version for compatibility checking.
    pub version: u32,
    /// Required Wasm features for this module.
    pub(crate) required_features: SerializedFeatures,
    /// Function types (deduplicated).
    pub(crate) func_types: Vec<SerializedFuncType>,
    /// Import declarations.
    pub(crate) imports: Vec<SerializedImport>,
    /// Internal functions with their metadata and instructions.
    pub(crate) internal_functions: Vec<SerializedInternalFunc>,
    /// Table types.
    pub(crate) tables: Vec<SerializedTableType>,
    /// Memory types.
    pub(crate) memories: Vec<SerializedMemoryType>,
    /// Global types and initial values.
    pub(crate) globals: Vec<SerializedGlobal>,
    /// Export declarations.
    pub(crate) exports: Vec<SerializedExport>,
    /// Start function index (if any).
    pub(crate) start: Option<u32>,
    /// The data segments
    pub(crate) data_segments: Vec<SerializedDataSegment>,
    /// The element segments
    pub(crate) element_segments: Vec<SerializedElementSegment>,

    /// The configuration of the engine that was used for the parsing
    pub(crate) engine_config: EngineConfig,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialization", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialization", derive(serde::Deserialize))]
pub(crate) struct EngineConfig {
    pub(crate) use_fuel: bool,
}
