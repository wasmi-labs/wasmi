// Invokes `$mac` with the execution handler's `extern "<abi>"` prepended.
// A callback macro since `macro_rules!` cannot expand to an ABI string.
#[cfg(all(feature = "unstable", not(feature = "stable")))]
macro_rules! invoke_with_handler_abi {
    ($mac:ident! { $($args:tt)* }) => {
        $mac! { extern "rust-preserve-none" $($args)* }
    };
}

#[cfg(all(
    not(all(feature = "unstable", not(feature = "stable"))),
    target_arch = "x86_64"
))]
macro_rules! invoke_with_handler_abi {
    ($mac:ident! { $($args:tt)* }) => {
        $mac! { extern "sysv64" $($args)* }
    };
}

#[cfg(all(
    not(all(feature = "unstable", not(feature = "stable"))),
    not(target_arch = "x86_64")
))]
macro_rules! invoke_with_handler_abi {
    ($mac:ident! { $($args:tt)* }) => {
        $mac! { extern "Rust" $($args)* }
    };
}

#[macro_use]
mod dispatch;
#[macro_use]
mod utils;
mod args;
mod cell;
mod exec;
mod func;
mod state;

use self::{
    args::Args,
    dispatch::{Break, Control},
    state::DoneReason,
};
pub use self::{
    cell::{
        Cell,
        CellError,
        CellsReader,
        CellsWriter,
        LiftFromCells,
        LiftFromCellsByValue,
        LoadByVal,
        LoadFromCellsByValue,
        LowerToCells,
        StoreToCells,
    },
    dispatch::{ExecutionOutcome, op_code_to_handler},
    func::{init_host_func_call, init_wasm_func_call, resume_wasm_func_call},
    state::{ExecContext, Inst, Stack},
};
