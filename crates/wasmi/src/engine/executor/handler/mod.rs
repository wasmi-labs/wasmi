#[macro_use]
mod dispatch;
#[macro_use]
mod utils;
mod cell;
mod eval;
mod exec;
mod func;
mod state;

pub use self::{
    cell::{
        load_from_cells,
        load_from_cells_into,
        store_to_cells,
        Cell,
        CellError,
        CellsReader,
        CellsWriter,
        LoadFromCells,
        StoreToCells,
    },
    dispatch::{op_code_to_handler, ExecutionOutcome},
    func::{init_host_func_call, init_wasm_func_call, resume_wasm_func_call},
    state::{Inst, Stack},
};
use self::{
    dispatch::{Break, Control, Done},
    state::DoneReason,
};
pub use self::{
    dispatch::{ExecutionOutcome, op_code_to_handler},
    func::{init_host_func_call, init_wasm_func_call, resume_wasm_func_call},
    state::{Inst, Stack},
};
