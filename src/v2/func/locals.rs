use crate::ValueType;

/// The local variables of a Wasm function instance.
#[derive(Debug)]
pub struct Locals {
    locals: Box<[Local]>,
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
#[derive(Debug)]
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
            locals: self.locals.into_boxed_slice(),
        }
    }
}
