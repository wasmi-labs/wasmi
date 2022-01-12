pub mod nan_preserving_float;

pub use nan_preserving_float::{F32, F64};

#[cfg(feature = "virtual_memory")]
mod vmem;

#[cfg(feature = "virtual_memory")]
pub use self::vmem::{VirtualMemory, VirtualMemoryError};
