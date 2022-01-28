use wasmi_core::ValueType;

/// The value stack that is emulated during Wasm to `wasmi` bytecode translation.
#[derive(Debug, Default)]
pub struct ValueStack {
    values: Vec<ValueType>,
}

impl ValueStack {
    /// Pushes the [`ValueType`] to the emulated [`ValueStack`].
    ///
    /// # Note
    ///
    /// In this [`ValueStack`] we push [`ValueType`] instead of [`Value`]
    /// to the stack since we are just emulating the Wasm [`ValueStack`] during
    /// translation from Wasm bytecode to `wasmi` bytecode.
    ///
    /// [`Value`]: [`wasmi_core::Value`]
    pub fn push(&mut self, value_type: ValueType) {
        self.values.push(value_type)
    }

    /// Pops the top most [`ValueType`] from the emulated [`ValueStack`].
    ///
    /// # Panics
    ///
    /// If the emulated [`ValueStack`] is empty.
    fn pop1(&mut self) -> ValueType {
        self.values
            .pop()
            .expect("tried to pop value from an empty emulated value stack")
    }

    /// Pops the 2 top most [`ValueType`] from the emulated [`ValueStack`].
    ///
    /// # Panics
    ///
    /// If the emulated [`ValueStack`] is empty.
    fn pop2(&mut self) -> (ValueType, ValueType) {
        let rhs = self.pop1();
        let lhs = self.pop1();
        (lhs, rhs)
    }

    /// Pops the 3 top most [`ValueType`] from the emulated [`ValueStack`].
    ///
    /// # Panics
    ///
    /// If the emulated [`ValueStack`] is empty.
    fn pop3(&mut self) -> (ValueType, ValueType, ValueType) {
        let v2 = self.pop1();
        let v1 = self.pop1();
        let v0 = self.pop1();
        (v0, v1, v2)
    }

    /// Returns the current length of the emulated [`ValueStack`].
    fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns `true` if the emulated [`ValueStack`] is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
