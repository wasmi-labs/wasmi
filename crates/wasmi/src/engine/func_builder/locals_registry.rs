/// A registry where local variables of a function are registered and resolved.
///
/// # Note
///
/// Note that in WebAssembly function parameters are also local variables.
///
/// The locals registry efficiently registers and resolves local variables.
/// The problem is that the Wasm specification allows to encode up to `u32::MAX`
/// local variables in a small and constant space via the binary encoding.
/// Therefore we need a way to efficiently cope with this worst-case scenario
/// in order to protect the `wasmi` interpreter against exploitations.
///
/// This implementation allows to access local variables in this worst-case
/// scenario with a worst time complexity of O(log n) and space requirement
/// of O(m + n) where n is the number of registered groups of local variables
/// and m is the number of actually used local variables.
///
/// Besides that local variable usages are cached to further minimize potential
/// exploitation impact.
#[derive(Debug, Default)]
pub struct LocalsRegistry {
    /// The amount of registered local variables.
    len_registered: u32,
}

impl LocalsRegistry {
    /// Returns the number of registered local variables.
    ///
    /// # Note
    ///
    /// Since in WebAssembly function parameters are also local variables
    /// this function actually returns the amount of function parameters
    /// and explicitly defined local variables.
    pub fn len_registered(&self) -> u32 {
        self.len_registered
    }

    /// Registers an `amount` of local variables.
    ///
    /// # Panics
    ///
    /// If too many local variables have been registered.
    pub fn register_locals(&mut self, amount: u32) {
        if amount == 0 {
            return;
        }
        self.len_registered = self.len_registered.checked_add(amount).unwrap_or_else(|| {
            panic!(
                "tried to register too many local variables for function: got {}, additional {amount}",
                self.len_registered
            )
        });
    }
}
