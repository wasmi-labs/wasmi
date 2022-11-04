pub mod snapshots;

pub use snapshots::preview_1::define_wasi;
pub use wasi_common::{Error, WasiDir, WasiFile};
pub use wasmi::Linker;
pub use wasmtime_wasi::{
    clocks,
    dir::Dir,
    file::{filetype_from, get_fd_flags, File},
    net,
    sched,
    stdio,
    WasiCtx,
    WasiCtxBuilder,
};

#[allow(dead_code)]
/// WasiCtxBuilder exists in case we wish to have our own versions of `WasiCtx`'s `rng` and `sched`
/// and possibly add our own methods as needed. It's private for now because I've found no use for it yet
struct WasmiWasiCtxBuilder(WasiCtx);

impl WasmiWasiCtxBuilder {}
