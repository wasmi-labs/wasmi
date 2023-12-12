use ::core::iter;

use super::{RegisterAlloc, TypedValue};
use crate::{engine::bytecode::Register, Error};
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
    /// A register referring to a preservation allocated register.
    Preserved(Register),
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
    /// Indicates whether to use `locals` to store `locals` on the provider stack.
    ///
    /// # Note
    ///
    /// This is an optimization since using [`LocalRefs`] is expensive.
    /// We mainly use [`LocalRefs`] to mitigate some attack surfaces of malicious inputs.
    /// We flip this `bool` flag once `providers` grow beyond a certain threshold.
    /// This way linear operations on `providers` can be seen as constant and
    /// thus not attackable.
    use_locals: bool,
    /// Used to store indices of `local.get` on the `providers` stack.
    locals: LocalRefs,
}

impl ProviderStack {
    /// Resets the [`ProviderStack`].
    pub fn reset(&mut self) {
        self.providers.clear();
        self.use_locals = false;
        self.locals.reset();
    }

    /// Preserves `local.get` on the [`ProviderStack`] by shifting to the preservation space.
    ///
    /// In case there are `local.get n` with `n == preserve_index` on the [`ProviderStack`]
    /// there is a [`Register`] on the preservation space allocated for them. The [`Register`]
    /// allocated this way is returned. Otherwise `None` is returned.
    pub fn preserve_locals(
        &mut self,
        preserve_index: u32,
        reg_alloc: &mut RegisterAlloc,
    ) -> Result<Option<Register>, Error> {
        /// Maximum provider stack height before switching to attack-immune
        /// [`LocalRefs`] implementation for `local.get` preservation.
        const THRESHOLD: usize = 16;

        if !self.use_locals && self.providers.len() >= THRESHOLD {
            self.sync_local_refs()
        }
        let local = i16::try_from(preserve_index)
            .map(Register::from_i16)
            .unwrap_or_else(|_| {
                panic!("encountered invalid local register index: {preserve_index}")
            });
        match self.use_locals {
            false => self.preserve_locals_inplace(local, reg_alloc),
            true => self.preserve_locals_extern(local, reg_alloc),
        }
    }

    /// Synchronizes [`LocalRefs`] with the current state of the `providers` stack.
    ///
    /// This is required to initialize usage of the attack-immune [`LocalRefs`] before first use.
    fn sync_local_refs(&mut self) {
        self.use_locals = true;
        for (index, provider) in self.providers.iter().enumerate() {
            let TaggedProvider::Local(local) = provider else {
                continue;
            };
            self.locals.push_at(*local, index);
        }
        self.use_locals = true;
    }

    /// Preserves the `local` [`Register`] on the provider stack in-place.
    ///
    /// # Note
    ///
    /// - This is the efficient case which is susceptible to malicious inputs
    ///   since it needs to iterate over the entire provider stack and might be
    ///   called roughly once per Wasm instruction in the worst-case.
    /// - Therefore we only use it behind a safety guard to remove the attack surface.
    fn preserve_locals_inplace(
        &mut self,
        local: Register,
        reg_alloc: &mut RegisterAlloc,
    ) -> Result<Option<Register>, Error> {
        debug_assert!(!self.use_locals);
        let mut preserved = None;
        for provider in &mut self.providers {
            let TaggedProvider::Local(register) = provider else {
                continue;
            };
            if *register != local {
                continue;
            }
            let preserved_register = match preserved {
                None => {
                    let register = reg_alloc.push_preserved()?;
                    preserved = Some(register);
                    register
                }
                Some(register) => {
                    reg_alloc.bump_preserved(register);
                    register
                }
            };
            *provider = TaggedProvider::Preserved(preserved_register);
        }
        Ok(preserved)
    }

    /// Preserves the `local` [`Register`] on the provider stack out-of-place.
    ///
    /// # Note
    ///
    /// - This is the inefficient case which is immune to malicious inputs
    ///   since it only iterates over the locals required for preservation
    ///   which are stored out-of-place of the provider stack.
    /// - Since this is slower we only use it when necessary.
    fn preserve_locals_extern(
        &mut self,
        local: Register,
        reg_alloc: &mut RegisterAlloc,
    ) -> Result<Option<Register>, Error> {
        debug_assert!(self.use_locals);
        let mut preserved = None;
        for provider_index in self.locals.drain_at(local) {
            let provider = &mut self.providers[provider_index];
            debug_assert!(matches!(provider, TaggedProvider::Local(_)));
            let preserved_register = match preserved {
                Some(register) => {
                    reg_alloc.bump_preserved(register);
                    register
                }
                None => {
                    let register = reg_alloc.push_preserved()?;
                    preserved = Some(register);
                    register
                }
            };
            *provider = TaggedProvider::Preserved(preserved_register);
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
        if self.use_locals {
            self.locals.push_at(reg, index);
        }
    }

    /// Pushes a dynamically allocated [`Register`] to the [`ProviderStack`].
    pub fn push_dynamic(&mut self, reg: Register) {
        debug_assert!(!reg.is_const());
        self.push(TaggedProvider::Dynamic(reg));
    }

    /// Pushes a preservation allocated [`Register`] to the [`ProviderStack`].
    pub fn push_preserved(&mut self, reg: Register) {
        debug_assert!(!reg.is_const());
        self.push(TaggedProvider::Preserved(reg));
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
            if self.use_locals {
                // If a `local.get` was popped from the provider stack we
                // also need to pop it from the `local.get` indices stack.
                let stack_index = self.locals.pop_at(register);
                debug_assert_eq!(self.providers.len(), stack_index);
            }
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
