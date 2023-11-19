use ::core::iter;

use super::{RegisterAlloc, TypedValue};
use crate::engine::{regmach::bytecode::Register, TranslationError};
use alloc::vec::Vec;
use smallvec::SmallVec;

#[cfg(doc)]
use wasmi_core::UntypedValue;

/// Tagged providers are inputs to `wasmi` bytecode instructions.
///
/// Either a [`Register`] or a constant [`UntypedValue`].
#[derive(Debug, Copy, Clone)]
pub enum TaggedProvider {
    /// A register referring to a function local constant value.
    ConstLocal(Register),
    /// A register referring to a function parameter or local variable.
    Local(Register),
    /// A register referring to a dynamically allocated register.
    Dynamic(Register),
    /// A register referring to a storage allocated register.
    Storage(Register),
    /// An untyped constant value.
    ConstValue(TypedValue),
}

/// The stack of providers.
///
/// # Note
///
/// This partially emulates the Wasm value stack during Wasm translation phase.
#[derive(Debug, Default)]
pub struct ProviderStack {
    /// The internal stack of providers.
    providers: Vec<TaggedProvider>,
    /// The indices of `local.get` providers on the [`ProviderStack`].
    locals: LocalRefs,
}

impl ProviderStack {
    /// Resets the [`ProviderStack`].
    pub fn reset(&mut self) {
        self.providers.clear();
        self.locals.reset();
    }

    /// Preserves `local.get` on the [`ProviderStack`] by shifting to storage space.
    ///
    /// In case there are `local.get n` with `n == preserve_index` on the [`ProviderStack`]
    /// there is a [`Register`] on the storage space allocated for them. The [`Register`]
    /// allocated this way is returned. Otherwise `None` is returned.
    pub fn preserve_locals(
        &mut self,
        preserve_index: u32,
        reg_alloc: &mut RegisterAlloc,
    ) -> Result<Option<Register>, TranslationError> {
        let mut preserved = None;
        let local = Register::from_i16(i16::try_from(preserve_index).unwrap_or_else(|_| {
            panic!("encountered invalid local register index: {preserve_index}")
        }));
        for provider_index in self.locals.drain_at(local) {
            let provider = &mut self.providers[provider_index];
            debug_assert!(matches!(provider, TaggedProvider::Local(_)));
            let preserved_register = match preserved {
                Some(register) => {
                    reg_alloc.bump_storage(register);
                    register
                }
                None => {
                    let register = reg_alloc.push_storage()?;
                    preserved = Some(register);
                    register
                }
            };
            *provider = TaggedProvider::Storage(preserved_register);
        }
        Ok(preserved)
    }

    /// Registers an `amount` of function inputs or local variables.
    ///
    /// # Errors
    ///
    /// If too many registers have been registered.
    pub fn register_locals(&mut self, amount: u32) {
        self.locals.register_locals(amount)
    }

    /// Returns the number of [`TaggedProvider`] on the [`ProviderStack`].
    pub fn len(&self) -> usize {
        self.providers.len()
    }

    /// Pushes a provider to the [`ProviderStack`].
    fn push(&mut self, provider: TaggedProvider) -> usize {
        let index = self.providers.len();
        self.providers.push(provider);
        index
    }

    /// Pushes a [`Register`] to the [`ProviderStack`] referring to a function parameter or local variable.
    pub fn push_local(&mut self, reg: Register) {
        debug_assert!(!reg.is_const());
        let index = self.push(TaggedProvider::Local(reg));
        self.locals.push_at(reg, index);
    }

    /// Pushes a dynamically allocated [`Register`] to the [`ProviderStack`].
    pub fn push_dynamic(&mut self, reg: Register) {
        debug_assert!(!reg.is_const());
        self.push(TaggedProvider::Dynamic(reg));
    }

    /// Pushes a storage allocated [`Register`] to the [`ProviderStack`].
    pub fn push_storage(&mut self, reg: Register) {
        debug_assert!(!reg.is_const());
        self.push(TaggedProvider::Storage(reg));
    }

    /// Pushes a [`Register`] to the [`ProviderStack`] referring to a function parameter or local variable.
    pub fn push_const_local(&mut self, reg: Register) {
        debug_assert!(reg.is_const());
        self.push(TaggedProvider::ConstLocal(reg));
    }

    /// Pushes a constant value to the [`ProviderStack`].
    pub fn push_const_value<T>(&mut self, value: T)
    where
        T: Into<TypedValue>,
    {
        self.push(TaggedProvider::ConstValue(value.into()));
    }

    /// Pops the top-most [`TaggedProvider`] from the [`ProviderStack`].
    ///
    /// # Panics
    ///
    /// If the [`ProviderStack`] is empty.
    pub fn peek(&self) -> TaggedProvider {
        self.providers
            .last()
            .copied()
            .unwrap_or_else(|| panic!("tried to peek provider from empty provider stack"))
    }

    /// Pops the top-most [`TaggedProvider`] from the [`ProviderStack`].
    ///
    /// # Panics
    ///
    /// If the [`ProviderStack`] is empty.
    pub fn pop(&mut self) -> TaggedProvider {
        let popped = self
            .providers
            .pop()
            .unwrap_or_else(|| panic!("tried to pop provider from empty provider stack"));
        if let TaggedProvider::Local(register) = popped {
            // If a `local.get` was popped from the provider stack we
            // also need to pop it from the `local.get` indices stack.
            let stack_index = self.locals.pop_at(register);
            debug_assert_eq!(self.providers.len(), stack_index);
        }
        popped
    }

    /// Peeks the `n` top-most [`TaggedProvider`] items of the [`ProviderStack`].
    ///
    /// # Panics
    ///
    /// If the [`ProviderStack`] does not contain at least `n` [`TaggedProvider`] items.
    pub fn peek_n(&self, n: usize) -> &[TaggedProvider] {
        let len = self.providers.len();
        assert!(
            n <= len,
            "tried to peek {n} items from provider stack with only {len} items"
        );
        &self.providers[(len - n)..]
    }
}

impl<'a> IntoIterator for &'a mut ProviderStack {
    type Item = &'a mut TaggedProvider;
    type IntoIter = core::slice::IterMut<'a, TaggedProvider>;

    fn into_iter(self) -> Self::IntoIter {
        self.providers.iter_mut()
    }
}

/// The index of a `local.get` on the [`ProviderStack`].
type StackIndex = usize;

#[derive(Debug, Default)]
pub struct LocalRefs {
    /// The indices of all `local.get` on the [`ProviderStack`] of all local variables.
    locals: Vec<SmallVec<[StackIndex; 2]>>,
}

impl LocalRefs {
    /// Resets the [`LocalRefs`].
    pub fn reset(&mut self) {
        self.locals.clear()
    }

    /// Registers an `amount` of function inputs or local variables.
    ///
    /// # Errors
    ///
    /// If too many registers have been registered.
    pub fn register_locals(&mut self, amount: u32) {
        self.locals
            .extend(iter::repeat_with(SmallVec::default).take(amount as StackIndex));
    }

    /// Returns the [`ProviderStack`] `local.get` indices of the `local` variable.
    ///
    /// # Panics
    ///
    /// If the `local` index is out of bounds.
    fn get_indices_mut(&mut self, local: Register) -> &mut SmallVec<[StackIndex; 2]> {
        debug_assert!(!local.is_const());
        &mut self.locals[local.to_i16().unsigned_abs() as usize]
    }

    /// Pushes the stack index of a `local.get` on the [`ProviderStack`].
    ///
    /// # Panics
    ///
    /// If the `local` index is out of bounds.
    pub fn push_at(&mut self, local: Register, stack_index: StackIndex) {
        self.get_indices_mut(local).push(stack_index);
    }

    /// Pops the stack index of a `local.get` on the [`ProviderStack`].
    ///
    /// # Panics
    ///
    /// - If the `local` index is out of bounds.
    /// - If there is no `local.get` stack index on the stack.
    pub fn pop_at(&mut self, local: Register) -> StackIndex {
        self.get_indices_mut(local).pop().unwrap_or_else(|| {
            panic!("missing stack index for local on the provider stack: {local:?}")
        })
    }

    /// Drains all `local.get` indices of the `local` variable on the [`ProviderStack`].
    ///
    /// # Panics
    ///
    /// If the `local` index is out of bounds.
    pub fn drain_at(&mut self, local: Register) -> smallvec::Drain<[StackIndex; 2]> {
        self.get_indices_mut(local).drain(..)
    }
}
