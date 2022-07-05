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
    func::{DisplayFuncIdx, DisplayFuncType},
    utils::{DisplaySequence, DisplaySlice},
};
pub use self::{func::DisplayFunc, instr::DisplayExecInstruction};
use super::{EngineInner, EngineResources};
use crate::{engine::ExecInstruction, Func, Instance, StoreContext};

impl EngineInner {
    /// Returns a [`Display`] wrapper to pretty print the given function.
    ///
    /// # Note
    ///
    /// This functionality is primarily for debugging purposes.
    ///
    /// [`Display`]: [`core::fmt::Display`]
    pub fn display_func<'ctx, 'engine, T>(
        &'engine self,
        ctx: StoreContext<'ctx, T>,
        func: Func,
    ) -> DisplayFunc<'ctx, 'engine, T> {
        DisplayFunc::new(ctx, self, func)
    }
}

impl ExecInstruction {
    /// Returns a [`Display`] wrapper to pretty print the given instruction.
    ///
    /// # Note
    ///
    /// This functionality is primarily for debugging purposes.
    ///
    /// [`Display`]: [`core::fmt::Display`]
    pub fn display_instr<'ctx, 'engine, T>(
        &self,
        ctx: StoreContext<'ctx, T>,
        instance: Instance,
        res: &'engine EngineResources,
    ) -> DisplayExecInstruction<'ctx, 'engine, T> {
        DisplayExecInstruction::new(ctx, res, instance, self)
    }
}
