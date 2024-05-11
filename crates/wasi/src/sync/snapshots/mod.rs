pub mod preview_1;

use wasi_common::{
    snapshots::preview_1::wasi_snapshot_preview1::{UserErrorConversion, WasiSnapshotPreview1},
    Error,
};

/// Implemented by Wasmi [`Linker`] and [`LinkerBuilder`] to populate them with WASI definitions.
///
/// [`Linker`]: wasmi::Linker
/// [`LinkerBuilder`]: wasmi::LinkerBuilder
pub trait AddWasi<T> {
    /// Add Wasi preview1 definitions to `self`.
    fn add_wasi<U>(
        &mut self,
        wasi_ctx: impl Fn(&mut T) -> &mut U + Send + Sync + Copy + 'static,
    ) -> Result<(), Error>
    where
        U: WasiSnapshotPreview1 + UserErrorConversion;
}
