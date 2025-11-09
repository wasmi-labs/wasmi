#[macro_use]
mod dispatch;
#[macro_use]
mod utils;
mod eval;
mod exec;
mod state;

#[cfg(feature = "trampolines")]
use self::dispatch::ControlContinue;
pub use self::{
    dispatch::{init_wasm_func_call, op_code_to_handler, ExecutionOutcome},
    state::{Inst, Stack},
};
use self::{
    dispatch::{Break, Control, ControlBreak, Done},
    state::DoneReason,
};
