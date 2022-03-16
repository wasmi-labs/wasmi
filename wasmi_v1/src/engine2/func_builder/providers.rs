use crate::Engine;

use super::LocalsRegistry;
use core::cmp::max;
use alloc::vec::Drain;
use wasmi_core::{Value, ValueType};
use super::super::Provider as ExecProvider;
use super::super::bytecode::Register as ExecRegister;

/// A stack of provided inputs for constructed instructions.
#[derive(Debug, Default)]
pub struct Providers {
    locals: LocalsRegistry,
    providers: Vec<Provider>,
    stacks: Stacks,
}

#[derive(Debug, Default)]
pub struct Stacks {
    len_dynamic: usize,
    max_dynamic: usize,
    len_preserved: usize,
    max_preserved: usize,
}

impl Stacks {
    pub fn max_dynamic(&self) -> usize {
        self.max_dynamic
    }

    fn bump_dynamic(&mut self) -> Register {
        let register = Register::Dynamic(self.len_dynamic);
        self.len_dynamic += 1;
        self.max_dynamic = max(self.max_dynamic, self.len_dynamic);
        register
    }

    fn bump_preserved(&mut self) -> Register {
        let register = Register::Preserved(self.len_preserved);
        self.len_preserved += 1;
        self.max_preserved = max(self.max_preserved, self.len_preserved);
        register
    }

    fn pop(&mut self, popped: &Provider) {
        match popped {
            Provider::Register(Register::Dynamic(_)) => {
                debug_assert!(self.len_dynamic > 0);
                self.len_dynamic -= 1;
            }
            Provider::Register(Register::Preserved(_)) => {
                debug_assert!(self.len_preserved > 0);
                self.len_preserved -= 1;
            }
            _ => (),
        }
    }
}

impl Providers {
    pub fn compile_provider(&self, engine: &Engine, provider: Provider) -> ExecProvider {
        match provider {
            Provider::Register(register) => {
                ExecProvider::from_register(self.compile_register(register))
            }
            Provider::Immediate(value) => {
    ExecProvider::from_immediate(engine.alloc_const(value))
            }
        }
    }

    pub fn compile_register(&self, register: Register) -> ExecRegister {
        let index = match register {
            Register::Local(index) => index,
            Register::Dynamic(index) => self.locals.len_registered() as usize + index,
            Register::Preserved(index) => {
                self.locals.len_registered() as usize + self.stacks.max_dynamic() + index
            }
        };
        let bounded = index.try_into().unwrap_or_else(|error| {
            panic!(
                "encountered out of bounds register index ({}): {}",
                index, error
            )
        });
        ExecRegister::from_inner(bounded)
    }


    /// Registers the `amount` of locals with their shared [`ValueType`].
    ///
    /// # Panics
    ///
    /// If too many local variables have been registered.
    pub fn register_locals(&mut self, value_type: ValueType, amount: u32) {
        self.locals.register_locals(value_type, amount)
    }

    pub fn push_local(&mut self, local_id: u32) -> Register {
        assert!(local_id < self.locals.len_registered());
        let register = Register::Local(local_id as usize);
        self.providers.push(register.into());
        register
    }

    pub fn push_dynamic(&mut self) -> Register {
        let register = self.stacks.bump_dynamic();
        self.providers.push(register.into());
        register
    }

    pub fn push_preserved(&mut self) -> Register {
        let register = self.stacks.bump_preserved();
        self.providers.push(register.into());
        register
    }

    pub fn push_const(&mut self, value: Value) -> Provider {
        let provider = Provider::from(value);
        self.providers.push(provider);
        provider
    }

    /// Pops the last provider from the [`Providers`] stack.
    ///
    /// # Panics
    ///
    /// If the stack is empty.
    pub fn pop(&mut self) -> Provider {
        let popped = self
            .providers
            .pop()
            .unwrap_or_else(|| panic!("unexpected missing provider"));
        self.stacks.pop(&popped);
        popped
    }

    pub fn pop2(&mut self) -> (Provider, Provider) {
        let rhs = self.pop();
        let lhs = self.pop();
        (lhs, rhs)
    }

    pub fn pop_n(&mut self, depth: usize) -> Drain<Provider> {
        let max_index = self.len() as usize;
        debug_assert!(depth <= max_index);
        let min_index = max_index - depth;
        for provider in &self.providers[min_index..] {
            self.stacks.pop(provider);
        }
        self.providers.drain(min_index..)
    }

    /// Returns the current length of the emulated [`Providers`].
    pub fn len(&self) -> u32 {
        self.providers.len() as u32
    }

    /// Returns `true` if the emulated [`Providers`] is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Shrinks the [`Providers`] to the given height.
    ///
    /// # Panics
    ///
    /// If the [`Providers`] height already is below the height since this
    /// usually indicates a bug in the translation of the Wasm to `wasmi`
    /// bytecode procedures.
    pub fn shrink_to(&mut self, new_height: u32) {
        let current_height = self.len();
        assert!(
            new_height <= current_height,
            "tried to shrink the value stack of height {} to height {}",
            current_height,
            new_height
        );
        let new_height = usize::try_from(new_height).unwrap_or_else(|error| {
            panic!(
                "could not convert stack height from `u32` to `usize`: {}",
                error
            )
        });
        for popped in self.providers.drain(new_height..) {
            self.stacks.pop(&popped);
        }
    }
}

/// A register provider of any of the existing register space.
#[derive(Debug, Copy, Clone)]
pub enum Register {
    /// The provided value lives in the local register space.
    ///
    /// # Note
    ///
    /// This includes all function parameters and function local
    /// variables.
    Local(usize),
    /// The provided value lives in the dynamic register space.
    ///
    /// # Note
    ///
    /// Dynamic registers are introduced during translation of Wasm
    /// instructions to store intermediate computation results.
    Dynamic(usize),
    /// The provided value lives in the preserved register space.
    ///
    /// # Note
    ///
    /// Preserved registers are introduced during translation of Wasm
    /// instructions to store results of computations that are going to be
    /// used at a later point.
    ///
    /// The distinction between `Local`, `Dynamic` and `Preserved` is
    /// important since due to construction it is not possible to directly map
    /// a preserved [`Register`] to a [`Register`] at time of construction
    /// but only after a function construction is about to be finalized
    /// and no further instructions are to be inserted.
    ///
    /// At function construction finalization all `Preserved` providers
    /// will be offset by the maximum amount of dynamic registers used
    /// throughout the function. The dynamic register space is the union
    /// of function parameters, function locals and dynamically required
    /// function registers.
    Preserved(usize),
}

/// A provided input for instruction construction.
#[derive(Debug, Copy, Clone)]
pub enum Provider {
    /// The input is stored in a register.
    Register(Register),
    /// The input is an immediate constant value.
    Immediate(Value),
}

impl From<Register> for Provider {
    fn from(register: Register) -> Self {
        Self::Register(register)
    }
}

impl From<Value> for Provider {
    fn from(value: Value) -> Self {
        Self::Immediate(value)
    }
}

impl Provider {
    /// Returns `Some` if the [`RegisterOrImmediate`] is a `Register`.
    pub fn filter_register(&self) -> Option<Register> {
        match self {
            Provider::Register(register) => Some(*register),
            Provider::Immediate(_) => None,
        }
    }

    /// Returns `Some` if the [`RegisterOrImmediate`] is an immediate value.
    pub fn filter_immediate(&self) -> Option<Value> {
        match self {
            Provider::Register(_) => None,
            Provider::Immediate(value) => Some(*value),
        }
    }
}

/// A reference to a provider slice.
///
/// # Note
///
/// References of this type are not deduplicated.
/// The actual registers of the [`RegisterSlice`] are
/// stored in a  [`RegisterSliceArena`].
/// Use [`RegisterSliceArena::resolve`] to access the
/// registers of the [`RegisterSlice`].
#[derive(Debug, Copy, Clone)]
pub struct ProviderSlice {
    /// The index to the first [`Register`] of the slice.
    first: u32,
    /// The number of registers in the slice.
    len: u32,
}

/// An arena to efficiently allocate provider slices.
///
/// # Note
///
/// This implementation does not deduplicate equivalent register slices.
#[derive(Debug, Default)]
pub struct ProviderSliceArena {
    providers: Vec<Provider>,
}

impl ProviderSliceArena {
    /// Allocates a new [`RegisterSlice`] consisting of the given registers.
    pub fn alloc<T>(&mut self, registers: T) -> ProviderSlice
    where
        T: IntoIterator<Item = Provider>,
    {
        let first = self.providers.len();
        self.providers.extend(registers);
        let len = self.providers.len() - first;
        let first = first
            .try_into()
            .unwrap_or_else(|error| panic!("out of bounds index for register slice: {}", first));
        let len = len
            .try_into()
            .unwrap_or_else(|error| panic!("register slice too long: {}", len));
        ProviderSlice { first, len }
    }

    /// Resolves a [`RegisterSlice`] to its underlying registers or immediates.
    pub fn resolve(&self, slice: ProviderSlice) -> &[Provider] {
        let first = slice.first as usize;
        let len = slice.len as usize;
        &self.providers[first..first + len]
    }

    /// Removes all previously allocated register slices from this arena.
    pub fn clear(&mut self) {
        self.providers.clear()
    }
}
