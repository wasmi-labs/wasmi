#[cfg(wasmi_use_unstable_features)]
macro_rules! invoke_with_handler_abi {
    ($mac:ident! { extern _ $($args:tt)* }) => {
        $mac! { extern "rust-preserve-none" $($args)* }
    };
}

#[cfg(all(not(wasmi_use_unstable_features), target_arch = "x86_64"))]
macro_rules! invoke_with_handler_abi {
    ($mac:ident! { extern _ $($args:tt)* }) => {
        $mac! { extern "sysv64" $($args)* }
    };
}

#[cfg(all(not(wasmi_use_unstable_features), not(target_arch = "x86_64")))]
macro_rules! invoke_with_handler_abi {
    ($mac:ident! { extern _ $($args:tt)* }) => {
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
