//! Re-export the commonly used wasi-cap-std-sync crate here. This saves
//! consumers of this library from having to keep additional dependencies
//! in sync.

pub mod snapshots;

pub use wasi_common::sync::*;

#[doc(inline)]
pub use self::snapshots::preview_1::{
    add_wasi_snapshot_preview1_to_linker as add_to_linker,
    AddWasi,
};

#[doc(inline)]
#[expect(deprecated)]
pub use self::snapshots::preview_1::add_wasi_snapshot_preview1_to_linker_builder as add_to_linker_builder;
