use crate::engine::{BrTable, RelativeDepth};
use core::slice;

/// A thin-wrapper around a Wasm `br_table` construct.
///
/// # ToDo
///
/// The current implementation of the [`WasmBrTable`] requires heap allocation
/// unfortunately. This is due to the fact that the API of `wasmparser`'s
/// `BrTable::targets` does not return a concrete iterator type that we can
/// thinnly wrap.
///
/// # Note
///
/// This wrapper assumes that Wasm validation of the `br_table` has already
/// happened and therefore panics upon encountering errors instead of forwarding
/// those.
#[derive(Debug)]
pub struct WasmBrTable {
    /// The default branch relative depth.
    default: RelativeDepth,
    /// The relative depths of the non-default targets.
    targets: Box<[RelativeDepth]>,
}

impl WasmBrTable {
    /// Creates a new thin-wrapper [`WasmBrTable`] around the `wasmparser` `br_table`.
    pub fn new(br_table: wasmparser::BrTable) -> Self {
        let default = RelativeDepth::from_u32(br_table.default());
        let targets = br_table
            .targets()
            .map(|relative_depth| {
                relative_depth.unwrap_or_else(|error| {
                    panic!(
                        "encountered unexpected invalid relative depth for target: {}",
                        error
                    )
                })
            })
            .map(RelativeDepth::from_u32)
            .collect::<Box<_>>();
        Self { default, targets }
    }
}

impl<'a> BrTable for &'a WasmBrTable {
    type Targets = WasmBrTableTargets<'a>;

    fn len(&self) -> u32 {
        self.targets.len() as u32
    }

    fn is_empty(&self) -> bool {
        self.targets.is_empty()
    }

    fn default(&self) -> RelativeDepth {
        self.default
    }

    fn targets(&self) -> Self::Targets {
        WasmBrTableTargets {
            iter: self.targets.iter(),
        }
    }
}

/// Thin wrapper around the targets of a `br_table` coming from `wasmparser`.
///
/// # Note
///
/// This wrapper assumes that Wasm validation of the `br_table` has already
/// happened and therefore panics upon encountering errors instead of forwarding
/// those.
pub struct WasmBrTableTargets<'a> {
    iter: slice::Iter<'a, RelativeDepth>,
}

impl<'a> Iterator for WasmBrTableTargets<'a> {
    type Item = RelativeDepth;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().copied()
    }
}
