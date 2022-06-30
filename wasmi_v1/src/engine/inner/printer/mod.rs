//! Definitions for pretty printing `wasmi` bytecode.
//!
//! The printed format is not intended to be used for (de)serialization.
//! The primary use case of this format is for debugging purposes.

mod bytecode;
mod func;
mod instr;
mod utils;

use self::{
    bytecode::{
        DisplayExecProvider,
        DisplayExecProviderSlice,
        DisplayExecRegister,
        DisplayExecRegisterSlice,
        DisplayGlobal,
        DisplayTarget,
    },
    func::{DisplayFunc, DisplayFuncIdx, DisplayFuncType},
    instr::DisplayExecInstruction,
    utils::{DisplaySequence, DisplaySlice},
};
use super::EngineInner;
use crate::{AsContext, Func};

impl EngineInner {
    /// Prints the given function in a human readable fashion.
    ///
    /// # Note
    ///
    /// This functionality is primarily for debugging purposes.
    pub fn print_func(&self, ctx: impl AsContext, func: Func) {
        println!("{}", DisplayFunc::new(ctx.as_context(), self, func));
    }
}
