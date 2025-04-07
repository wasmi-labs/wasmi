mod access;
mod buffer;
mod error;
mod ty;

pub use self::{
    access::{
        load,
        load_at,
        load_extend,
        load_extend_at,
        store,
        store_at,
        store_wrap,
        store_wrap_at,
        ExtendInto,
    },
    error::MemoryError,
    ty::{MemoryType, MemoryTypeBuilder},
};
