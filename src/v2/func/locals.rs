#![allow(dead_code)]

use crate::ValueType;
use alloc::sync::Arc;

/// The local variables of a Wasm function instance.
#[derive(Debug, Clone)]
pub struct Locals {
    // We are using an `Arc` instead of a `Box` here to make `clone` cheap.
    // We currently need cloning to prevent `unsafe` Rust usage when calling
    // a Wasm or host function.
    locals: Arc<[Local]>,
}

impl Locals {
    /// Builds up a new local variable group.
    pub fn build() -> LocalsBuilder {
        LocalsBuilder { locals: Vec::new() }
    }

    /// Returns the local variable group definitions as slice in order.
    pub fn as_slice(&self) -> &[Local] {
        &self.locals
    }
}

/// A local variables builder.
#[derive(Debug)]
pub struct LocalsBuilder {
    /// Partially constructed local variables.
    locals: Vec<Local>,
}

/// A group of local variables in a Wasm function.
#[derive(Debug, Copy, Clone)]
pub struct Local {
    value_type: ValueType,
    amount: usize,
}

impl Local {
    /// A Wasm local variable group definition.
    pub fn new(value_type: ValueType, amount: usize) -> Self {
        Self { value_type, amount }
    }

    /// Returns the shared value type of the local variable group.
    pub fn value_type(&self) -> ValueType {
        self.value_type
    }

    /// Returns the amount of local variables in the locals group.
    pub fn amount(&self) -> usize {
        self.amount
    }
}

impl LocalsBuilder {
    /// Push another local variables group to the locals builder.
    pub fn push_local(mut self, value_type: ValueType, amount: usize) -> Self {
        self.locals.push(Local::new(value_type, amount));
        self
    }

    /// Finishes constructing local variable group definition.
    pub fn finish(self) -> Locals {
        Locals {
            locals: self.locals.into(),
        }
    }
}
