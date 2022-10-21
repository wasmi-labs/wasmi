use core::cmp;

/// The current height of the emulated Wasm value stack.
#[derive(Debug, Default, Copy, Clone)]
pub struct ValueStackHeight {
    /// The current height of the emulated value stack of the translated function.
    ///
    /// # Note
    ///
    /// This does not include input parameters and local variables.
    height: u32,
    /// The maximum height of the emulated value stack of the translated function.
    ///
    /// # Note
    ///
    /// This does not include input parameters and local variables.
    max_height: u32,
}

impl ValueStackHeight {
    /// Returns the current length of the emulated value stack.
    ///
    /// # Note
    ///
    /// This does not include input parameters and local variables.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Returns the maximum value stack height.
    ///
    /// # Note
    ///
    /// This does not include input parameters and local variables.
    pub fn max_stack_height(&self) -> u32 {
        self.max_height
    }

    /// Updates the pinned maximum value stack height.
    fn update_max_height(&mut self) {
        self.max_height = cmp::max(self.height, self.max_height);
    }

    /// Pushes an `amount` of values to the emulated value stack.
    pub fn push_n(&mut self, amount: u32) {
        self.height += amount;
        self.update_max_height();
    }

    /// Pushes a value to the emulated value stack.
    pub fn push(&mut self) {
        self.push_n(1)
    }

    /// Pops an `amount` of elements from the emulated value stack.
    pub fn pop_n(&mut self, amount: u32) {
        debug_assert!(amount <= self.height);
        self.height -= amount;
    }

    /// Pops 1 element from the emulated value stack.
    pub fn pop1(&mut self) {
        self.pop_n(1)
    }

    /// Pops 2 elements from the emulated value stack.
    pub fn pop2(&mut self) {
        self.pop_n(2)
    }

    /// Pops 3 elements from the emulated value stack.
    pub fn pop3(&mut self) {
        self.pop_n(3)
    }

    /// Shrinks the emulated value stack to the given height.
    ///
    /// # Panics
    ///
    /// If the value stack height already is below the height since this
    /// usually indicates a bug in the translation of the Wasm to `wasmi`
    /// bytecode procedures.
    pub fn shrink_to(&mut self, new_height: u32) {
        assert!(new_height <= self.height);
        self.height = new_height;
    }
}
