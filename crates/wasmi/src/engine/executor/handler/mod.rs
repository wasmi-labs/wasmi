#[macro_use]
mod dispatch;
#[macro_use]
mod utils;
mod args;
mod cell;
mod exec;
mod func;
mod state;

#[cfg(not(feature = "indirect-dispatch"))]
pub use self::dispatch::op_code_to_handler;
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
    dispatch::ExecutionOutcome,
    func::{init_host_func_call, init_wasm_func_call, resume_wasm_func_call},
    state::{ExecContext, Inst, Stack},
};
