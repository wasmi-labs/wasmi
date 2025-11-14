#[macro_use]
mod dispatch;
#[macro_use]
mod utils;
mod eval;
mod exec;
mod func;
mod state;

#[cfg(feature = "portable-dispatch")]
use self::dispatch::ControlContinue;
pub use self::{
    dispatch::{op_code_to_handler, ExecutionOutcome},
    func::{init_host_func_call, init_wasm_func_call, resume_wasm_func_call},
    state::{Inst, Stack},
};
use self::{
    dispatch::{Break, Control, ControlBreak, Done},
    state::DoneReason,
};
