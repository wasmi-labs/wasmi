#[macro_use]
mod dispatch;
#[macro_use]
mod utils;
mod eval;
mod exec;
mod state;

pub use self::{
    dispatch::{init_wasm_func_call, op_code_to_handler},
    state::{Inst, Stack},
};
use self::{
    dispatch::{Control, ControlBreak, ControlContinue, Done},
    state::DoneReason,
};
