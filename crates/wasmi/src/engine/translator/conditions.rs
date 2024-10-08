//! Wasmi translation post-conditions.
//!
//! These are run if `cfg(debug-assertions)` are enabled and
//! provide another layer of protection for invalid Wasmi translations.
//!
//! They are mostly intended for debugging purposes or for fuzzing Wasmi
//! as they would add too much overhead to conventional Wasmi translation.
//!
//! The set of post-conditions is run right after Wasmi translation has
//! finished compiling a Wasm function and just before the compiled function
//! is stored into Wasmi's `CodeMap`.
//!
//! The checks generally check invariants and guarantees given by the Wasmi
//! translation and therefore also act as "documentation" about some of them.

use super::{FuncTranslator, Instr};
use crate::ir::{self, BranchOffset, Instruction, Reg};
use core::{
    cmp,
    fmt::{self, Display},
};

/// An error describing a broken Wasmi translation invariant.
#[derive(Debug)]
pub struct Error {
    /// The erraneous `Instruction` index.
    instr: Instr,
    /// The reason for the error.
    reason: Reason,
}

/// The reason behind a broken Wasmi translation invariant.
#[derive(Debug)]
pub enum Reason {
    InvalidRegister {
        /// The invalid `Reg`.
        reg: Reg,
    },
    InvalidGlobal {
        /// The invalid `Global` index.
        invalid_global: ir::index::Global,
    },
    InvalidFunc {
        /// The invalid `Global` index.
        invalid_func: ir::index::Func,
    },
    InvalidTable {
        /// The invalid `Table` index.
        invalid_table: ir::index::Table,
    },
    InvalidMemory {
        /// The invalid `Memory` index.
        invalid_memory: ir::index::Memory,
    },
    InvalidBranchOffset {
        /// The invalid `BranchOffset`.
        invalid_offset: BranchOffset,
    },
    InvalidBranchTarget {
        /// The invalid target of the branch `Instruction`.
        invalid_target: Instr,
    },
    InvalidConsumeFuel {
        /// The invalid fuel consumption amount. (usually 0)
        invalid_fuel: u64,
    },
    InvalidInstructionParameter {
        /// The invalid `Instruction` parameter index.
        invalid_param: Instr,
    },
    InvalidReturnValues {
        /// The invalid number of returned values.
        invalid_results: u32,
    },
}

impl Display for ErrorWithContext<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pos = self.pos();
        let instr = self.instr();
        match self.error.reason {
            Reason::InvalidRegister { reg } => {
                writeln!(
                    f,
                    "unexpected invalid register for instruction at {pos}:\n\
                        \t- instruction: {instr:?}\n\
                        \t- register: {reg:?}\
                    "
                )
            }
            Reason::InvalidGlobal { invalid_global } => {
                let len_globals = self.ctx.len_globals();
                write!(
                    f,
                    "unexpected invalid global for instruction at {pos}:\n\
                        \t- instruction: {instr:?}\n\
                        \t- invalid global: {invalid_global:?}\n\
                        \t- number of globals in Wasm module: {len_globals}\
                    "
                )
            }
            Reason::InvalidFunc { invalid_func } => {
                let len_funcs = self.ctx.len_funcs();
                write!(
                    f,
                    "unexpected invalid function for instruction at {pos}:\n\
                        \t- instruction: {instr:?}\n\
                        \t- invalid function: {invalid_func:?}\n\
                        \t- number of functions in Wasm module: {len_funcs}\
                    "
                )
            }
            Reason::InvalidTable { invalid_table } => {
                let len_tables = self.ctx.len_tables();
                write!(
                    f,
                    "unexpected invalid table for instruction at {pos}:\n\
                        \t- instruction: {instr:?}\n\
                        \t- invalid table: {invalid_table:?}\n\
                        \t- number of tables in Wasm module: {len_tables}\
                    "
                )
            }
            Reason::InvalidMemory { invalid_memory } => {
                let len_memories = self.ctx.len_memories();
                write!(
                    f,
                    "unexpected invalid linear memory for instruction at {pos}:\n\
                        \t- instruction: {instr:?}\n\
                        \t- invalid linear memory: {invalid_memory:?}\n\
                        \t- number of memories in Wasm module: {len_memories}\
                    "
                )
            }
            Reason::InvalidBranchOffset { invalid_offset } => {
                let fuel_metering_status = match self.ctx.is_fuel_metering_enabled() {
                    true => "enabled",
                    false => "disabled",
                };
                write!(
                    f,
                    "unexpected invalid branching offset for instruction at {pos}:\n\
                        \t- instruction: {instr:?}\n\
                        \t- invalid branch offset: {invalid_offset:?}\n\
                        \t- fuel metering: {fuel_metering_status}\
                    "
                )
            }
            Reason::InvalidBranchTarget { invalid_target } => {
                let invalid_target_pos = Self::instr_pos(invalid_target);
                let invalid_target = self.ctx.resolve_instr(invalid_target);
                let fuel_metering_status = match self.ctx.is_fuel_metering_enabled() {
                    true => "enabled",
                    false => "disabled",
                };
                let branch_kind = match invalid_target_pos.cmp(&pos) {
                    cmp::Ordering::Less => "backward",
                    cmp::Ordering::Equal => "self-loop",
                    cmp::Ordering::Greater => "forward",
                };
                write!(
                    f,
                    "unexpected invalid branching offset for instruction at {pos}:\n\
                        \t- instruction: {instr:?}\n\
                        \t- invalid target at: {invalid_target_pos:?}\n\
                        \t- invalid branch target: {invalid_target:?}\n\
                        \t- branch kind: {branch_kind}\n\
                        \t- fuel metering: {fuel_metering_status}\
                    "
                )
            }
            Reason::InvalidConsumeFuel { invalid_fuel } => {
                let fuel_metering_status = match self.ctx.is_fuel_metering_enabled() {
                    true => "enabled",
                    false => "disabled",
                };
                write!(
                    f,
                    "unexpected invalid fuel consumption for instruction at {pos}:\n\
                        \t- instruction: {instr:?}\n\
                        \t- invalid fuel consumption: {invalid_fuel:?}\n\
                        \t- fuel metering: {fuel_metering_status}\
                    "
                )
            }
            Reason::InvalidInstructionParameter { invalid_param } => {
                let invalid_param_pos = Self::instr_pos(invalid_param);
                write!(
                    f,
                    "unexpected invalid instruction parameter for instruction at {pos}:\n\
                        \t- instruction: {instr:?}\n\
                        \t- invalid instruction parameter at: {invalid_param_pos}\
                        \t- invalid instruction parameter: {invalid_param:?}\
                    "
                )
            }
            Reason::InvalidReturnValues { invalid_results } => {
                write!(
                    f,
                    "unexpected invalid instruction parameter for instruction at {pos}:\n\
                        \t- instruction: {instr:?}\n\
                        \t- invalid number of results: {invalid_results}\
                    "
                )
            }
        }
    }
}

/// An error combined with contextual information to improve its [`Display`] implementation.
pub struct ErrorWithContext<'ctx> {
    /// The contextual information for the error.
    ctx: ErrorContext<'ctx>,
    /// The underlying error.
    error: Error,
}

/// A context to populate an [`Error`] with more information for its [`Display`] implementation.
pub struct ErrorContext<'ctx> {
    /// The underlying Wasmi function translator that has already finished its job.
    translator: &'ctx FuncTranslator,
}

impl ErrorContext<'_> {
    /// Returns `true` if fuel metering is enabled for the translation.
    fn is_fuel_metering_enabled(&self) -> bool {
        self.translator.fuel_costs.is_some()
    }

    /// Resolves the [`Instruction`] at `instr`.
    ///
    /// # Panics
    ///
    /// If `instr` is invalid for the underlying Wasmi translator.
    fn resolve_instr(&self, instr: Instr) -> Instruction {
        *self.translator.alloc.instr_encoder.instrs.get(instr)
    }

    /// Returns the number of global variables for the Wasm module of the compiled function.
    fn len_globals(&self) -> usize {
        self.translator.module.len_globals()
    }

    /// Returns the number of functions for the Wasm module of the compiled function.
    fn len_funcs(&self) -> usize {
        self.translator.module.len_funcs()
    }

    /// Returns the number of tables for the Wasm module of the compiled function.
    fn len_tables(&self) -> usize {
        self.translator.module.len_tables()
    }

    /// Returns the number of linear memories for the Wasm module of the compiled function.
    fn len_memories(&self) -> usize {
        self.translator.module.len_memories()
    }
}

impl ErrorWithContext<'_> {
    /// Returns the `u32` position of `instr` within the instruction sequence.
    fn instr_pos(instr: Instr) -> u32 {
        u32::from(ir::index::Instr::from(instr))
    }

    /// Returns the `u32` position of the error's `instr` within the instruction sequence.
    fn pos(&self) -> u32 {
        Self::instr_pos(self.error.instr)
    }

    /// Resolves the error's `instr` to [`Instruction`].
    fn instr(&self) -> Instruction {
        self.ctx.resolve_instr(self.error.instr)
    }
}

/// Checks if the invariants of Wasmi function translation are uphold.
///
/// This function is called after Wasmi function translation has finished
/// and before the Wasmi engine's `CodeMap` is populated with the compiled
/// function.
///
/// The checking is designed to only be performed for builds with
/// `debug-assertions` enabled due to massive translation overhead.
///
/// # Errors
///
/// If a Wasmi translation invariant is broken.
pub fn verify_translation_invariants(translator: &FuncTranslator) -> Result<(), ErrorWithContext> {
    let checker = TranslationInvariantsChecker { translator };
    match checker.verify_translation_invariants() {
        Ok(_) => Ok(()),
        Err(error) => {
            let ctx = ErrorContext { translator };
            Err(ErrorWithContext { ctx, error })
        }
    }
}

/// Encapsulates state required for translation invariants checking.
struct TranslationInvariantsChecker<'translator> {
    /// The underlying Wasmi function translator which has already finished its job.
    translator: &'translator FuncTranslator,
}

impl TranslationInvariantsChecker<'_> {
    /// Checks if the invariants of Wasmi function translation are uphold.
    ///
    /// Read more here: [`verify_translation_invariants`]
    ///
    /// # Errors
    ///
    /// If a Wasmi translation invariant is broken.
    fn verify_translation_invariants(&self) -> Result<(), Error> {
        todo!()
    }
}
