use crate::engine::bytecode2::Register;
use alloc::vec::{Drain, Vec};
use wasmi_core::UntypedValue;

/// Providers are inputs to `wasmi` bytecode instructions.
///
/// Either a [`Register`] or a constant [`UntypedValue`].
#[derive(Debug, Copy, Clone)]
pub enum Provider {
    /// A register.
    Register(Register),
    /// An untyped constant value.
    Const(UntypedValue),
}

/// The stack of providers.
///
/// # Note
///
/// This partially emulates the Wasm value stack during Wasm translation phase.
#[derive(Debug, Default)]
pub struct ProviderStack {
    /// The internal stack of providers.
    providers: Vec<Provider>,
}

impl ProviderStack {
    /// Pushes a provider to the [`ProviderStack`].
    pub fn push(&mut self, provider: Provider) {
        self.providers.push(provider);
    }

    /// Pushes a [`Register`] to the [`ProviderStack`].
    pub fn push_register(&mut self, reg: Register) {
        self.push(Provider::Register(reg));
    }

    /// Pushes a constant value to the [`ProviderStack`].
    pub fn push_const<T>(&mut self, value: T)
    where
        T: Into<UntypedValue>,
    {
        self.push(Provider::Const(value.into()));
    }

    /// Pops the top-most [`Provider`] from the [`ProviderStack`].
    ///
    /// # Panics
    ///
    /// If the [`ProviderStack`] is empty.
    pub fn pop(&mut self) -> Provider {
        self.providers
            .pop()
            .unwrap_or_else(|| panic!("tried to pop provider from empty provider stack"))
    }

    /// Pops the two top-most [`Provider`] items from the [`ProviderStack`].
    ///
    /// # Panics
    ///
    /// If the [`ProviderStack`] does not contain at least two [`Provider`] items.
    pub fn pop2(&mut self) -> (Provider, Provider) {
        let rhs = self.pop();
        let lhs = self.pop();
        (lhs, rhs)
    }

    /// Pops the three top-most [`Provider`] items from the [`ProviderStack`].
    ///
    /// # Panics
    ///
    /// If the [`ProviderStack`] does not contain at least three [`Provider`] items.
    pub fn pop3(&mut self) -> (Provider, Provider, Provider) {
        let (snd, trd) = self.pop2();
        let fst = self.pop();
        (fst, snd, trd)
    }

    /// Pops the `n` top-most [`Provider`] items from the [`ProviderStack`].
    ///
    /// # Panics
    ///
    /// If the [`ProviderStack`] does not contain at least `n` [`Provider`] items.
    pub fn pop_n(&mut self, n: usize) -> Drain<Provider> {
        let len = self.providers.len();
        assert!(
            n <= len,
            "tried to pop {n} items from provider stack with only {len} items"
        );
        self.providers.drain(len - n..)
    }
}
