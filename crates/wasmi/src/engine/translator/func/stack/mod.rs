mod consts;
mod locals;
mod provider;
mod register_alloc;

pub use self::{
    consts::{FuncLocalConsts, FuncLocalConstsIter},
    locals::LocalRefs,
    provider::{ProviderStack, TaggedProvider},
    register_alloc::{RegisterAlloc, RegisterSpace},
};
use super::{comparator::AllocConst, PreservedLocal, TypedVal};
use crate::{
    core::UntypedVal,
    engine::{
        translator::func::{Provider, UntypedProvider},
        TranslationError,
    },
    ir::{Reg, RegSpan},
    Error,
};
use alloc::vec::Vec;

/// Typed inputs to Wasmi bytecode instructions.
///
/// Either a [`Reg`] or a constant [`UntypedVal`].
///
/// # Note
///
/// The [`TypedProvider`] is used primarily during translation of a Wasmi
/// function where types of constant values play an important role.
pub type TypedProvider = Provider<TypedVal>;

impl TypedProvider {
    /// Converts the [`TypedProvider`] to a resolved [`UntypedProvider`].
    pub fn into_untyped(self) -> UntypedProvider {
        match self {
            Self::Register(register) => UntypedProvider::Register(register),
            Self::Const(value) => UntypedProvider::Const(UntypedVal::from(value)),
        }
    }

    /// Creates a new [`TypedProvider::Register`].
    pub fn register(register: impl Into<Reg>) -> Self {
        Self::Register(register.into())
    }
}

impl From<TaggedProvider> for TypedProvider {
    fn from(provider: TaggedProvider) -> Self {
        match provider {
            TaggedProvider::Local(register)
            | TaggedProvider::Dynamic(register)
            | TaggedProvider::Preserved(register)
            | TaggedProvider::ConstLocal(register) => Self::Register(register),
            TaggedProvider::ConstValue(value) => Self::Const(value),
        }
    }
}

/// The value stack.
#[derive(Debug, Default)]
pub struct ValueStack {
    providers: ProviderStack,
    reg_alloc: RegisterAlloc,
    consts: FuncLocalConsts,
}

impl ValueStack {
    /// Resets the [`ValueStack`].
    pub fn reset(&mut self) {
        self.providers.reset();
        self.reg_alloc.reset();
        self.consts.reset();
    }

    /// Pops [`Provider`] from the [`ValueStack`] until it has the given stack `height`.
    pub fn trunc(&mut self, height: usize) {
        assert!(height <= self.height());
        while self.height() != height {
            self.drop();
        }
    }

    /// Preserves `local.get` on the [`ProviderStack`] by shifting to the preservation space.
    ///
    /// In case there are `local.get n` with `n == preserve_index` on the [`ProviderStack`]
    /// there is a [`Reg`] on the storage space allocated for them. The [`Reg`]
    /// allocated this way is returned. Otherwise `None` is returned.
    pub fn preserve_locals(&mut self, preserve_index: u32) -> Result<Option<Reg>, Error> {
        self.providers
            .preserve_locals(preserve_index, &mut self.reg_alloc)
    }

    /// Preserves all locals on the [`ProviderStack`] by shifting them to the preservation space.
    ///
    /// Calls `f(local_register, preserved_register)` for each `local_register` preserved this way with its
    /// newly allocated `preserved_register` on the presevation register space.
    pub fn preserve_all_locals(
        &mut self,
        f: impl FnMut(PreservedLocal) -> Result<(), Error>,
    ) -> Result<(), Error> {
        self.providers.preserve_all_locals(&mut self.reg_alloc, f)
    }

    /// Frees all preservation slots that have been flagged for removal.
    ///
    /// This is important to allow them for reuse for future preservations.
    pub fn gc_preservations(&mut self) {
        self.reg_alloc.gc_preservations()
    }

    /// Returns the number of [`Provider`] on the [`ValueStack`].
    ///
    /// # Note
    ///
    /// This is the same as the height of the [`ValueStack`].
    pub fn height(&self) -> usize {
        self.providers.len()
    }

    /// Returns the number of registers allocated by the [`RegisterAlloc`].
    pub fn len_registers(&self) -> u16 {
        // The addition won't overflow since both operands are in the range of `0..i16::MAX`.
        self.consts.len_consts() + self.reg_alloc.len_registers()
    }

    /// Registers an `amount` of function inputs or local variables.
    ///
    /// # Errors
    ///
    /// If too many registers have been registered.
    ///
    /// # Panics
    ///
    /// If the [`RegisterAlloc`] is not in its initialization phase.
    pub fn register_locals(&mut self, amount: u32) -> Result<(), Error> {
        self.providers.register_locals(amount);
        self.reg_alloc.register_locals(amount)?;
        Ok(())
    }

    /// Finishes the local variable registration phase.
    ///
    /// # Note
    ///
    /// After this operation no local variable can be registered anymore.
    /// However, it is then possible to push and pop dynamic and storage registers to the stack.
    pub fn finish_register_locals(&mut self) {
        self.reg_alloc.finish_register_locals()
    }

    /// Allocates a new function local constant value and returns its [`Reg`].
    ///
    /// # Note
    ///
    /// Constant values allocated this way are deduplicated and return shared [`Reg`].
    pub fn alloc_const<T>(&mut self, value: T) -> Result<Reg, Error>
    where
        T: Into<UntypedVal>,
    {
        self.consts.alloc(value.into())
    }

    /// Converts a [`TypedProvider`] into a [`Reg`].
    ///
    /// This allocates constant values for [`TypedProvider::Const`].
    pub fn provider2reg(&mut self, provider: &TypedProvider) -> Result<Reg, Error> {
        match provider {
            Provider::Register(register) => Ok(*register),
            Provider::Const(value) => self.alloc_const(*value),
        }
    }

    /// Returns the allocated function local constant values in reversed allocation order.
    ///
    /// # Note
    ///
    /// Upon calling a function all of its function local constant values are
    /// inserted into the current execution call frame in reversed allocation order
    /// and accessed via negative [`Reg`] index where the 0 index is referring
    /// to the first function local and the -1 index is referring to the first
    /// allocated function local constant value.
    pub fn func_local_consts(&self) -> FuncLocalConstsIter {
        self.consts.iter()
    }

    /// Pushes the given [`TypedProvider`] to the [`ValueStack`].
    ///
    /// # Note
    ///
    /// This is a convenice method for [`ValueStack::push_register`] and [`ValueStack::push_const`].
    pub fn push_provider(&mut self, provider: TypedProvider) -> Result<(), Error> {
        match provider {
            Provider::Register(register) => self.push_register(register)?,
            Provider::Const(value) => self.push_const(value),
        }
        Ok(())
    }

    /// Pushes a constant value to the [`ProviderStack`].
    pub fn push_const<T>(&mut self, value: T)
    where
        T: Into<TypedVal>,
    {
        self.providers.push_const_value(value)
    }

    /// Pushes the given [`Reg`] to the [`ValueStack`].
    ///
    /// # Errors
    ///
    /// If too many registers have been registered.
    pub fn push_register(&mut self, reg: Reg) -> Result<(), Error> {
        match self.reg_alloc.register_space(reg) {
            RegisterSpace::Dynamic => {
                self.reg_alloc.push_dynamic()?;
                self.providers.push_dynamic(reg);
                return Ok(());
            }
            RegisterSpace::Preserve => {
                // Note: we currently do not call `self.reg_alloc.push_storage()`
                //       since that API would push always another register on the preservation
                //       stack instead of trying to bump the amount of already existing
                //       preservation slots for the same register if possible.
                self.reg_alloc.bump_preserved(reg);
                self.providers.push_preserved(reg);
            }
            RegisterSpace::Local => {
                self.providers.push_local(reg);
            }
            RegisterSpace::Const => {
                self.providers.push_const_local(reg);
            }
        }
        Ok(())
    }

    /// Pushes a [`Reg`] to the [`ValueStack`] referring to a function parameter or local variable.
    ///
    /// # Errors
    ///
    /// If too many registers have been registered.
    pub fn push_local(&mut self, local_index: u32) -> Result<Reg, Error> {
        let reg = i16::try_from(local_index)
            .ok()
            .map(Reg::from)
            .filter(|reg| self.reg_alloc.is_local(*reg))
            .ok_or_else(|| Error::from(TranslationError::RegisterOutOfBounds))?;
        self.providers.push_local(reg);
        Ok(reg)
    }

    /// Pushes a dynamically allocated [`Reg`] to the [`ValueStack`].
    ///
    /// # Errors
    ///
    /// If too many registers have been registered.
    pub fn push_dynamic(&mut self) -> Result<Reg, Error> {
        let reg = self.reg_alloc.push_dynamic()?;
        self.providers.push_dynamic(reg);
        Ok(reg)
    }

    /// Drops the top-most [`Provider`] from the [`ValueStack`].
    pub fn drop(&mut self) {
        self.reg_alloc.pop_provider(self.providers.pop());
    }

    /// Pops the top-most [`Provider`] from the [`ValueStack`].
    ///
    /// Use [`Self::drop`] if you are not interested in the returned provider.
    #[must_use]
    pub fn pop(&mut self) -> TypedProvider {
        self.reg_alloc.pop_provider(self.providers.pop())
    }

    /// Peeks the top-most [`Provider`] from the [`ValueStack`].
    pub fn peek(&self) -> TypedProvider {
        TypedProvider::from(self.providers.peek())
    }

    /// Pops the two top-most [`Provider`] from the [`ValueStack`].
    pub fn pop2(&mut self) -> (TypedProvider, TypedProvider) {
        let rhs = self.pop();
        let lhs = self.pop();
        (lhs, rhs)
    }

    /// Pops the three top-most [`Provider`] from the [`ValueStack`].
    pub fn pop3(&mut self) -> (TypedProvider, TypedProvider, TypedProvider) {
        let (v1, v2) = self.pop2();
        let v0 = self.pop();
        (v0, v1, v2)
    }

    /// Popn the `n` top-most [`Provider`] from the [`ValueStack`] and store them in `result`.
    ///
    /// # Note
    ///
    /// - The top-most [`Provider`] will be the n-th item in `result`.
    /// - The `result` [`Vec`] will be cleared before refilled.
    pub fn pop_n(&mut self, n: usize, result: &mut Vec<TypedProvider>) {
        result.clear();
        for _ in 0..n {
            let provider = self.pop();
            result.push(provider);
        }
        result[..].reverse()
    }

    /// Removes the `n` top-most [`Provider`] from the [`ValueStack`].
    pub fn remove_n(&mut self, n: usize) {
        for _ in 0..n {
            self.drop();
        }
    }

    /// Peeks the `n` top-most [`Provider`] from the [`ValueStack`] and store them in `result`.
    ///
    /// # Note
    ///
    /// - The top-most [`Provider`] will be the n-th item in `result`.
    /// - The `result` [`Vec`] will be cleared before refilled.
    pub fn peek_n(&mut self, n: usize, result: &mut Vec<TypedProvider>) {
        result.clear();
        result.extend(
            self.providers
                .peek_n(n)
                .iter()
                .copied()
                .map(TypedProvider::from),
        );
    }

    /// Pushes the given `providers` into the [`ValueStack`].
    ///
    /// # Errors
    ///
    /// If too many registers have been registered.
    pub fn push_n(&mut self, providers: &[TypedProvider]) -> Result<(), Error> {
        for provider in providers {
            match *provider {
                TypedProvider::Register(register) => self.push_register(register)?,
                TypedProvider::Const(value) => self.push_const(value),
            }
        }
        Ok(())
    }

    /// Returns a [`RegSpan`] of `n` registers as if they were dynamically allocated.
    ///
    /// # Note
    ///
    /// - This procedure pushes dynamic [`Reg`] onto the [`ValueStack`].
    /// - This is primarily used to allocate branch parameters for control
    ///   flow frames such as Wasm `block`, `loop` and `if` as well as for
    ///   instructions that may return multiple values such as `call`.
    ///
    /// # Errors
    ///
    /// If this procedure would allocate more registers than are available.
    pub fn push_dynamic_n(&mut self, n: usize) -> Result<RegSpan, Error> {
        let registers = self.reg_alloc.push_dynamic_n(n)?;
        for register in registers.iter_sized(n) {
            self.providers.push_dynamic(register);
        }
        Ok(registers)
    }

    /// Returns a [`RegSpan`] of `n` registers as if they were dynamically allocated.
    ///
    /// # Note
    ///
    /// - This procedure does not push anything onto the [`ValueStack`].
    /// - This is primarily used to allocate branch parameters for control
    ///   flow frames such as Wasm `block`, `loop` and `if`.
    ///
    /// # Errors
    ///
    /// If this procedure would allocate more registers than are available.
    pub fn peek_dynamic_n(&mut self, n: usize) -> Result<RegSpan, Error> {
        let registers = self.reg_alloc.push_dynamic_n(n)?;
        self.reg_alloc.pop_dynamic_n(n);
        Ok(registers)
    }

    /// Finalizes register allocation and allows to defragment the register space.
    pub fn finalize_alloc(&mut self) {
        self.reg_alloc.finalize_alloc()
    }

    /// Returns the defragmented [`Reg`].
    pub fn defrag_register(&mut self, register: Reg) -> Reg {
        self.reg_alloc.defrag_register(register)
    }

    /// Returns the [`RegisterSpace`] for the given [`Reg`].
    pub fn get_register_space(&self, register: Reg) -> RegisterSpace {
        self.reg_alloc.register_space(register)
    }

    /// Increase preservation [`Reg`] usage.
    ///
    /// # Note
    ///
    /// - This is mainly used to extend the lifetime of `else` providers on the stack.
    /// - This does nothing if `register` is not a preservation [`Reg`].
    pub fn inc_register_usage(&mut self, register: Reg) {
        self.reg_alloc.inc_register_usage(register)
    }

    /// Decrease preservation [`Reg`] usage.
    ///
    /// # Note
    ///
    /// - This is mainly used to shorten the lifetime of `else` providers on the stack.
    /// - This does nothing if `register` is not a preservation [`Reg`].
    pub fn dec_register_usage(&mut self, register: Reg) {
        self.reg_alloc.dec_register_usage(register)
    }
}

impl AllocConst for ValueStack {
    fn alloc_const<T: Into<UntypedVal>>(&mut self, value: T) -> Result<Reg, Error> {
        self.consts.alloc(value.into())
    }
}
