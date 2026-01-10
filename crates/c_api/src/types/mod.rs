mod export;
mod r#extern;
mod func;
mod global;
mod import;
mod memory;
mod table;
mod val;

pub use self::{
    export::*,
    r#extern::*,
    func::*,
    global::*,
    import::*,
    memory::*,
    table::*,
    val::*,
};

/// Utility type representing minimum and maximum limitations for Wasm types.
///
/// # Examples
///
/// This could represent the minimum and maximum number of pages for a Wasm [`MemoryType`].
///
/// [`MemoryType`]: wasmi::MemoryType
#[repr(C)]
#[derive(Clone)]
pub struct wasm_limits_t {
    /// The minimum limit for the Wasm type.
    pub min: u32,
    /// The maximum limit for the Wasm type.
    pub max: u32,
}

impl wasm_limits_t {
    /// Returns the maximum limit if valid or `None`.
    ///
    /// A limit equal to `u32::MAX` is invalid.
    pub(crate) fn max(&self) -> Option<u32> {
        if self.max == u32::MAX {
            None
        } else {
            Some(self.max)
        }
    }
}
