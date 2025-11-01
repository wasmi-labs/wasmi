#[macro_use]
mod dispatch;
mod eval;
#[macro_use]
mod utils;
mod exec;
mod state;

use self::{dispatch::Done, state::DoneReason};
pub use self::{
    dispatch::{init_wasm_func_call, op_code_to_handler},
    state::{Inst, Stack},
};
