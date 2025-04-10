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
use crate::{
    core::UntypedVal,
    ir::{self, BranchOffset, ComparatorAndOffset, Instruction, Reg},
};
use core::{
    cmp,
    fmt::{self, Display},
};
use std::vec::Vec;

/// A non-empty list of [`Error`]s.
pub struct ErrorList<'ctx> {
    ctx: ErrorContext<'ctx>,
    errors: Vec<Error>,
}

impl<'ctx> ErrorList<'ctx> {
    /// Creates a new [`ErrorList`].
    ///
    /// # Panics
    ///
    /// If `errors` is empty.
    fn new(ctx: ErrorContext<'ctx>, errors: Vec<Error>) -> Self {
        assert!(!errors.is_empty());
        Self { ctx, errors }
    }
}

impl ErrorList<'_> {
    /// Returns `true` if `self` contains at least one [`Error`].
    fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Returns the number of [`Error`]s in `self`.
    fn len_errors(&self) -> usize {
        self.errors.len()
    }
}

impl Display for ErrorList<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.has_errors() {
            return writeln!(f, "all checked Wasmi translation invariants are uphold");
        }
        writeln!(
            f,
            "encountered {} broken invariants for Wasmi translation:",
            self.len_errors()
        )?;
        let ctx = self.ctx;
        for (n, error) in self.errors.iter().cloned().enumerate() {
            write!(f, "error({n}): {}", ErrorWithContext { ctx, error })?;
        }
        Ok(())
    }
}

/// An error describing a broken Wasmi translation invariant.
#[derive(Debug, Clone)]
pub struct Error {
    /// The erraneous `Instruction` index.
    instr: Instr,
    /// The reason for the error.
    reason: Reason,
}

impl Error {
    /// Creates a new [`Error`] with the `instr` and `reason`.
    pub fn new(instr: Instr, reason: Reason) -> Self {
        Self { instr, reason }
    }
}

/// The reason behind a broken Wasmi translation invariant.
#[derive(Debug, Clone)]
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
    MissingFunctionLocalConst {
        /// The register for which the function local constant is missing.
        reg: Reg,
    },
    InvalidComparatorAndOffset {
        /// The invalid encoded comparator and offset.
        value: UntypedVal,
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
            Reason::MissingFunctionLocalConst { reg } => {
                write!(
                    f,
                    "missing function local constant value for instruction at {pos}:\n\
                        \t- instruction: {instr:?}\n\
                        \t- register to constant: {reg:?}\
                    "
                )
            }
            Reason::InvalidComparatorAndOffset { value } => {
                write!(
                    f,
                    "invalid encoded comparator and offset for instruction at {pos}:\n\
                        \t- instruction: {instr:?}\n\
                        \t- encoded value: {value:?}\
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
#[derive(Copy, Clone)]
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
pub fn verify_translation_invariants(translator: &FuncTranslator) -> Result<(), ErrorList> {
    TranslationInvariantsChecker { translator }.verify_translation_invariants()
}

/// Extension convenience trait to push to errors accumulator upon `Err`.
trait OkOrPush {
    /// Pushes to `errors` if `self` is an `Err`.
    fn ok_or_push(self, errors: &mut Vec<Error>);
}

impl OkOrPush for Result<(), Error> {
    fn ok_or_push(self, errors: &mut Vec<Error>) {
        if let Err(e) = self {
            errors.push(e);
        }
    }
}

/// Encapsulates state required for translation invariants checking.
struct TranslationInvariantsChecker<'translator> {
    /// The underlying Wasmi function translator which has already finished its job.
    translator: &'translator FuncTranslator,
}

impl<'translator> TranslationInvariantsChecker<'translator> {
    /// Checks if the invariants of Wasmi function translation are uphold.
    ///
    /// Read more here: [`verify_translation_invariants`]
    ///
    /// # Errors
    ///
    /// If a Wasmi translation invariant is broken.
    fn verify_translation_invariants(&self) -> Result<(), ErrorList<'translator>> {
        let mut errors = Vec::new();
        let mut checker = NonZeroBranchOffsets::new(self.translator);
        let instrs = self.translator.alloc.instr_encoder.instrs.as_slice();
        checker.start().ok_or_push(&mut errors);
        for (current, instruction) in instrs.iter().enumerate() {
            let instr = Instr::from_usize(current);
            checker.instr(instr, instruction).ok_or_push(&mut errors);
        }
        checker.finish().ok_or_push(&mut errors);
        // At this point all invariants have been processed.
        if errors.is_empty() {
            return Ok(());
        }
        Err(ErrorList::new(
            ErrorContext {
                translator: self.translator,
            },
            errors,
        ))
    }
}

/// Implemented by all post conditions checker.
trait InvariantsChecker<'a> {
    /// Create a new [`InvariantsChecker`].
    fn new(translator: &'a FuncTranslator) -> Self;
    /// Setup the [`InvariantsChecker`].
    ///
    /// This method is called once before inspecting all instructions.
    fn start(&mut self) -> Result<(), Error> {
        Ok(())
    }
    /// Checks the `instruction` at `instr` with this [`InvariantsChecker`].
    fn instr(&mut self, instr: Instr, instruction: &Instruction) -> Result<(), Error>;
    /// Finishes the [`InvariantsChecker`].
    ///
    /// This method is called once after inspecting all instructions.
    fn finish(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

/// Checks that all branch offsets are non-zero when fuel metering is enabled.
pub struct NonZeroBranchOffsets<'a> {
    translator: &'a FuncTranslator,
    /// True if fuel metering is enabled for fast access.
    consume_fuel: bool,
}

impl<'a> InvariantsChecker<'a> for NonZeroBranchOffsets<'a> {
    fn new(translator: &'a FuncTranslator) -> Self {
        let consume_fuel = translator.engine.config().get_consume_fuel();
        Self {
            translator,
            consume_fuel,
        }
    }

    fn instr(&mut self, instr: Instr, instruction: &Instruction) -> Result<(), Error> {
        use Instruction as I;
        if !self.consume_fuel {
            return Ok(());
        }
        let offset = match *instruction {
            | I::Branch { offset } => offset,
            | I::BranchCmpFallback { params, .. } => {
                let Some(value) = self.translator.alloc.stack.resolve_const(params) else {
                    return Err(Error::new(
                        instr,
                        Reason::MissingFunctionLocalConst { reg: params },
                    ));
                };
                let Some(decoded) = ComparatorAndOffset::from_untyped(value) else {
                    return Err(Error::new(
                        instr,
                        Reason::InvalidComparatorAndOffset { value },
                    ));
                };
                decoded.offset
            }
            | I::BranchI32And { offset, .. }
            | I::BranchI32AndImm16 { offset, .. }
            | I::BranchI32Or { offset, .. }
            | I::BranchI32OrImm16 { offset, .. }
            | I::BranchI32Xor { offset, .. }
            | I::BranchI32XorImm16 { offset, .. }
            | I::BranchI32AndEqz { offset, .. }
            | I::BranchI32AndEqzImm16 { offset, .. }
            | I::BranchI32OrEqz { offset, .. }
            | I::BranchI32OrEqzImm16 { offset, .. }
            | I::BranchI32XorEqz { offset, .. }
            | I::BranchI32XorEqzImm16 { offset, .. }
            | I::BranchI32Eq { offset, .. }
            | I::BranchI32EqImm16 { offset, .. }
            | I::BranchI32Ne { offset, .. }
            | I::BranchI32NeImm16 { offset, .. }
            | I::BranchI32LtS { offset, .. }
            | I::BranchI32LtSImm16Lhs { offset, .. }
            | I::BranchI32LtSImm16Rhs { offset, .. }
            | I::BranchI32LtU { offset, .. }
            | I::BranchI32LtUImm16Lhs { offset, .. }
            | I::BranchI32LtUImm16Rhs { offset, .. }
            | I::BranchI32LeS { offset, .. }
            | I::BranchI32LeSImm16Lhs { offset, .. }
            | I::BranchI32LeSImm16Rhs { offset, .. }
            | I::BranchI32LeU { offset, .. }
            | I::BranchI32LeUImm16Lhs { offset, .. }
            | I::BranchI32LeUImm16Rhs { offset, .. }
            | I::BranchI64Eq { offset, .. }
            | I::BranchI64EqImm16 { offset, .. }
            | I::BranchI64Ne { offset, .. }
            | I::BranchI64NeImm16 { offset, .. }
            | I::BranchI64LtS { offset, .. }
            | I::BranchI64LtSImm16Lhs { offset, .. }
            | I::BranchI64LtSImm16Rhs { offset, .. }
            | I::BranchI64LtU { offset, .. }
            | I::BranchI64LtUImm16Lhs { offset, .. }
            | I::BranchI64LtUImm16Rhs { offset, .. }
            | I::BranchI64LeS { offset, .. }
            | I::BranchI64LeSImm16Lhs { offset, .. }
            | I::BranchI64LeSImm16Rhs { offset, .. }
            | I::BranchI64LeU { offset, .. }
            | I::BranchI64LeUImm16Lhs { offset, .. }
            | I::BranchI64LeUImm16Rhs { offset, .. }
            | I::BranchF32Eq { offset, .. }
            | I::BranchF32Ne { offset, .. }
            | I::BranchF32Lt { offset, .. }
            | I::BranchF32Le { offset, .. }
            | I::BranchF64Eq { offset, .. }
            | I::BranchF64Ne { offset, .. }
            | I::BranchF64Lt { offset, .. }
            | I::BranchF64Le { offset, .. } => BranchOffset::from(offset),
            _ => return Ok(()),
        };
        if offset.to_i32() != 0 {
            return Ok(());
        }
        Err(Error::new(
            instr,
            Reason::InvalidBranchOffset {
                invalid_offset: offset,
            },
        ))
    }
}
