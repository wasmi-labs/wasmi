mod host_error;
mod nan_preserving_float;

#[cfg(feature = "virtual_memory")]
mod vmem;

#[cfg(feature = "virtual_memory")]
pub use self::vmem::{VirtualMemory, VirtualMemoryError};

pub use self::{
    host_error::HostError,
    nan_preserving_float::{F32, F64},
};
