#[macro_use]
mod dispatch;
#[macro_use]
mod utils;
mod eval;
mod exec;
mod state;

use self::{dispatch::Done, state::DoneReason};
pub use self::{
    dispatch::{
        init_wasm_func_call,
        op_code_to_handler,
        ControlFlowBreak,
        ControlFlowContinue,
    },
    state::{Inst, Stack},
};
