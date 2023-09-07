use super::TypedValue;
use crate::engine::regmach::bytecode::Register;
use alloc::vec::{Drain, Vec};

#[cfg(doc)]
use wasmi_core::UntypedValue;

/// Tagged providers are inputs to `wasmi` bytecode instructions.
///
/// Either a [`Register`] or a constant [`UntypedValue`].
#[derive(Debug, Copy, Clone)]
pub enum TaggedProvider {
    /// A register referring to a function parameter or local variable.
    Local(Register),
    /// A register referring to a dynamically allocated register.
    Dynamic(Register),
    /// A register referring to a storage allocated register.
    Storage(Register),
    /// An untyped constant value.
    Const(TypedValue),
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
}

impl ProviderStack {
    /// Resets the [`ProviderStack`].
    pub fn reset(&mut self) {
        self.providers.clear()
    }

    /// Returns the number of [`TaggedProvider`] on the [`ProviderStack`].
    pub fn len(&self) -> usize {
        self.providers.len()
    }

    /// Pushes a provider to the [`ProviderStack`].
    fn push(&mut self, provider: TaggedProvider) {
        self.providers.push(provider);
    }

    /// Pushes a [`Register`] to the [`ProviderStack`] referring to a function parameter or local variable.
    pub fn push_local(&mut self, reg: Register) {
        self.push(TaggedProvider::Local(reg));
    }

    /// Pushes a dynamically allocated [`Register`] to the [`ProviderStack`].
    pub fn push_dynamic(&mut self, reg: Register) {
        self.push(TaggedProvider::Dynamic(reg));
    }

    /// Pushes a storage allocated [`Register`] to the [`ProviderStack`].
    pub fn push_storage(&mut self, reg: Register) {
        self.push(TaggedProvider::Storage(reg));
    }

    /// Pushes a constant value to the [`ProviderStack`].
    pub fn push_const<T>(&mut self, value: T)
    where
        T: Into<TypedValue>,
    {
        self.push(TaggedProvider::Const(value.into()));
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
        self.providers
            .pop()
            .unwrap_or_else(|| panic!("tried to pop provider from empty provider stack"))
    }

    /// Pops the `n` top-most [`TaggedProvider`] items from the [`ProviderStack`].
    ///
    /// # Panics
    ///
    /// If the [`ProviderStack`] does not contain at least `n` [`TaggedProvider`] items.
    pub fn pop_n(&mut self, n: usize) -> Drain<TaggedProvider> {
        let len = self.providers.len();
        assert!(
            n <= len,
            "tried to pop {n} items from provider stack with only {len} items"
        );
        self.providers.drain((len - n)..)
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
