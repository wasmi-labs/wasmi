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
    cell::{read_cells, write_cells, Cell, ReadCell, WriteCell},
    dispatch::{op_code_to_handler, ExecutionOutcome},
    func::{init_host_func_call, init_wasm_func_call, resume_wasm_func_call},
    state::{Inst, Stack},
};
use self::{
    dispatch::{Break, Control, Done},
    state::DoneReason,
};
