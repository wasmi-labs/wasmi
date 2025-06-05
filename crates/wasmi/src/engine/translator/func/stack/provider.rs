use super::{LocalRefs, RegisterAlloc, TypedVal};
use crate::{engine::translator::func::PreservedLocal, ir::Reg, Error};
use alloc::vec::Vec;
use arrayvec::ArrayVec;

#[cfg(doc)]
use crate::core::UntypedVal;

/// Tagged providers are inputs to Wasmi bytecode instructions.
///
/// Either a [`Reg`] or a constant [`UntypedVal`].
#[derive(Debug, Copy, Clone)]
pub enum TaggedProvider {
    /// A register referring to a function local constant value.
    ConstLocal(Reg),
    /// A register referring to a function parameter or local variable.
    Local(Reg),
    /// A register referring to a dynamically allocated register.
    Dynamic(Reg),
    /// A register referring to a preservation allocated register.
    Preserved(Reg),
    /// An untyped constant value.
    ConstValue(TypedVal),
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
    /// The number of locals on the `providers` stack.
    ///
    /// # Note
    ///
    /// This is used to act as fast bailout for local preservation in the common
    /// case that no locals are on the stack.
    len_locals: usize,
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
        self.len_locals = 0;
    }

    /// Maximum provider stack height before switching to attack-immune
    /// [`LocalRefs`] implementation for `local.get` preservation.
    const PRESERVE_THRESHOLD: usize = 16;

    /// Synchronizes [`LocalRefs`] with the current state of the `providers` stack.
    ///
    /// This is required to initialize usage of the attack-immune [`LocalRefs`] before first use.
    fn sync_local_refs(&mut self) {
        if self.use_locals || self.providers.len() < Self::PRESERVE_THRESHOLD {
            return;
        }
        for (index, provider) in self.providers.iter().enumerate() {
            let TaggedProvider::Local(local) = provider else {
                continue;
            };
            self.locals.push_at(*local, index);
        }
        self.use_locals = true;
    }

    /// Preserves `local.get` on the [`ProviderStack`] by shifting to the preservation space.
    ///
    /// In case there are `local.get n` with `n == preserve_index` on the [`ProviderStack`]
    /// there is a [`Reg`] on the preservation space allocated for them. The [`Reg`]
    /// allocated this way is returned. Otherwise `None` is returned.
    pub fn preserve_locals(
        &mut self,
        preserve_index: u32,
        reg_alloc: &mut RegisterAlloc,
    ) -> Result<Option<Reg>, Error> {
        if self.len_locals == 0 {
            return Ok(None);
        }
        self.sync_local_refs();
        let local = i16::try_from(preserve_index)
            .map(Reg::from)
            .unwrap_or_else(|_| {
                panic!("encountered invalid local register index: {preserve_index}")
            });
        match self.use_locals {
            false => self.preserve_locals_inplace(local, reg_alloc),
            true => self.preserve_locals_extern(local, reg_alloc),
        }
    }

    /// Preserves all locals on the [`ProviderStack`] by shifting them to the preservation space.
    ///
    /// # Note
    ///
    /// - Calls `f(local_index, preserved_register)` for each local preserved this way with its `local_index`
    ///   and the newly allocated `preserved_register` on the presevation register space.
    /// - If the same local appears multiple  times on the provider stack `f` is only called once
    ///   representing all of them.
    pub fn preserve_all_locals(
        &mut self,
        reg_alloc: &mut RegisterAlloc,
        f: impl FnMut(PreservedLocal) -> Result<(), Error>,
    ) -> Result<(), Error> {
        if self.len_locals == 0 {
            return Ok(());
        }
        self.sync_local_refs();
        match self.use_locals {
            false => self.preserve_all_locals_inplace(reg_alloc, f),
            true => self.preserve_all_locals_extern(reg_alloc, f),
        }
    }

    /// Preserves the `local` [`Reg`] on the provider stack in-place.
    ///
    /// # Note
    ///
    /// - This is the efficient case which is susceptible to malicious inputs
    ///   since it needs to iterate over the entire provider stack and might be
    ///   called roughly once per Wasm instruction in the worst-case.
    /// - Therefore we only use it behind a safety guard to remove the attack surface.
    fn preserve_locals_inplace(
        &mut self,
        local: Reg,
        reg_alloc: &mut RegisterAlloc,
    ) -> Result<Option<Reg>, Error> {
        debug_assert!(!self.use_locals);
        let mut preserved = None;
        for provider in &mut self.providers {
            let TaggedProvider::Local(register) = *provider else {
                continue;
            };
            debug_assert!(reg_alloc.is_local(register));
            if register != local {
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
            self.len_locals -= 1;
            if self.len_locals == 0 {
                break;
            }
        }
        Ok(preserved)
    }

    /// Preserves all locals on the [`ProviderStack`] by shifting them to the preservation space.
    ///
    /// # Note
    ///
    /// - Calls `f(local_index, preserved_register)` for each local preserved this way with its `local_index`
    ///   and the newly allocated `preserved_register` on the presevation register space.
    /// - If the same local appears multiple  times on the provider stack `f` is only called once
    ///   representing all of them.
    ///
    /// # Dev. Note
    ///
    /// - This is the efficient case which is susceptible to malicious inputs
    ///   since it needs to iterate over the entire provider stack and might be
    ///   called roughly once per Wasm instruction in the worst-case.
    /// - Therefore we only use it behind a safety guard to remove the attack surface.
    fn preserve_all_locals_inplace(
        &mut self,
        reg_alloc: &mut RegisterAlloc,
        mut f: impl FnMut(PreservedLocal) -> Result<(), Error>,
    ) -> Result<(), Error> {
        debug_assert!(!self.use_locals);
        let mut preserved = <ArrayVec<PreservedLocal, { Self::PRESERVE_THRESHOLD }>>::new();
        for provider in self.providers.iter_mut().rev() {
            let TaggedProvider::Local(local_register) = *provider else {
                continue;
            };
            debug_assert!(reg_alloc.is_local(local_register));
            let (preserved_register, is_new) =
                // Note: linear search is fine since we operate only on very small sets of data.
                match preserved.iter().find(|p| p.local == local_register) {
                    Some(preserved_local) => {
                        let preserved_register = preserved_local.preserved;
                        reg_alloc.bump_preserved(preserved_register);
                        (preserved_register, false)
                    }
                    None => {
                        let preserved_register = reg_alloc.push_preserved()?;
                        preserved.push(PreservedLocal::new(local_register, preserved_register));
                        (preserved_register, true)
                    }
                };
            *provider = TaggedProvider::Preserved(preserved_register);
            self.len_locals -= 1;
            if is_new {
                f(PreservedLocal::new(local_register, preserved_register))?;
            }
            if self.len_locals == 0 {
                break;
            }
        }
        debug_assert_eq!(self.len_locals, 0);
        Ok(())
    }

    /// Preserves the `local` [`Reg`] on the provider stack out-of-place.
    ///
    /// # Note
    ///
    /// - This is the inefficient case which is immune to malicious inputs
    ///   since it only iterates over the locals required for preservation
    ///   which are stored out-of-place of the provider stack.
    /// - Since this is slower we only use it when necessary.
    fn preserve_locals_extern(
        &mut self,
        local: Reg,
        reg_alloc: &mut RegisterAlloc,
    ) -> Result<Option<Reg>, Error> {
        debug_assert!(self.use_locals);
        let mut preserved = None;
        self.locals.drain_at(local, |provider_index| {
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
            self.len_locals -= 1;
            Ok(())
        })?;
        Ok(preserved)
    }

    /// Preserves all locals on the [`ProviderStack`] by shifting them to the preservation space.
    ///
    /// # Note
    ///
    /// - Calls `f(local_index, preserved_register)` for each local preserved this way with its `local_index`
    ///   and the newly allocated `preserved_register` on the presevation register space.
    /// - If the same local appears multiple  times on the provider stack `f` is only called once
    ///   representing all of them.
    ///
    /// # Dev. Note
    ///
    /// - This is the inefficient case which is immune to malicious inputs
    ///   since it only iterates over the locals required for preservation
    ///   which are stored out-of-place of the provider stack.
    /// - Since this is slower we only use it when necessary.
    fn preserve_all_locals_extern(
        &mut self,
        reg_alloc: &mut RegisterAlloc,
        mut f: impl FnMut(PreservedLocal) -> Result<(), Error>,
    ) -> Result<(), Error> {
        debug_assert!(self.use_locals);
        let mut group = None;
        self.locals.drain_all(|local, index| {
            let preserved = match group {
                Some((group, preserved)) if group == local => {
                    reg_alloc.bump_preserved(preserved);
                    preserved
                }
                _ => {
                    let preserved = reg_alloc.push_preserved()?;
                    group = Some((local, preserved));
                    f(PreservedLocal::new(local, preserved))?;
                    preserved
                }
            };
            self.providers[index] = TaggedProvider::Preserved(preserved);
            self.len_locals -= 1;
            Ok(())
        })?;
        debug_assert_eq!(self.len_locals, 0);
        Ok(())
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
    #[inline(always)]
    fn push(&mut self, provider: TaggedProvider) {
        let index = self.providers.len();
        self.providers.push(provider);
        if let TaggedProvider::Local(reg) = provider {
            self.len_locals += 1;
            if self.use_locals {
                self.locals.push_at(reg, index);
            }
        }
    }

    /// Pushes a [`Reg`] to the [`ProviderStack`] referring to a function parameter or local variable.
    pub fn push_local(&mut self, reg: Reg) {
        debug_assert!(!reg.is_const());
        self.push(TaggedProvider::Local(reg));
    }

    /// Pushes a dynamically allocated [`Reg`] to the [`ProviderStack`].
    pub fn push_dynamic(&mut self, reg: Reg) {
        debug_assert!(!reg.is_const());
        self.push(TaggedProvider::Dynamic(reg));
    }

    /// Pushes a preservation allocated [`Reg`] to the [`ProviderStack`].
    pub fn push_preserved(&mut self, reg: Reg) {
        debug_assert!(!reg.is_const());
        self.push(TaggedProvider::Preserved(reg));
    }

    /// Pushes a [`Reg`] to the [`ProviderStack`] referring to a function parameter or local variable.
    pub fn push_const_local(&mut self, reg: Reg) {
        debug_assert!(reg.is_const());
        self.push(TaggedProvider::ConstLocal(reg));
    }

    /// Pushes a constant value to the [`ProviderStack`].
    pub fn push_const_value<T>(&mut self, value: T)
    where
        T: Into<TypedVal>,
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
            self.len_locals -= 1;
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
