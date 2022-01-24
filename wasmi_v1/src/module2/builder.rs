use super::Module;
use crate::FuncType;

/// A builder for a WebAssembly [`Module`].
#[derive(Debug)]
pub struct ModuleBuilder {
    func_types: Vec<FuncType>,
}

impl Default for ModuleBuilder {
    fn default() -> Self {
        Self {
            func_types: Vec::new(),
        }
    }
}

impl ModuleBuilder {
    /// Reserves enough space for at least `hint` [`FuncType`] instances
    /// in the [`Module`] under construction.
    ///
    /// # Note
    ///
    /// This procedure serves as a memory allocation optimization.
    pub fn reserve_func_types(&mut self, hint: u32) {
        self.func_types.reserve_exact(hint as usize);
    }

    /// Pushes the given [`FuncType`] to the [`Module`] under construction.
    ///
    /// Returns the raw `u32` index to the pushed [`FuncType`].
    pub fn push_func_type(&mut self, func_type: FuncType) -> u32 {
        let index = u32::try_from(self.func_types.len())
            .unwrap_or_else(|error| panic!("encountered out of bounds function types: {}", error));
        self.func_types.push(func_type);
        index
    }

    /// Finishes construction of the WebAssembly [`Module`].
    pub fn finish(self) -> Module {
        todo!()
    }
}
