mod provider;
mod register_alloc;

pub use self::{
    provider::{ProviderStack, TaggedProvider},
    register_alloc::{DefragRegister, RegisterAlloc},
};
use crate::engine::{
    bytecode2::Register,
    func_builder::TranslationErrorInner,
    Instr,
    TranslationError,
};
use wasmi_core::UntypedValue;

/// Tagged providers are inputs to `wasmi` bytecode instructions.
///
/// Either a [`Register`] or a constant [`UntypedValue`].
#[derive(Debug, Copy, Clone)]
pub enum Provider {
    /// A register.
    Register(Register),
    /// An untyped constant value.
    Const(UntypedValue),
}

/// The value stack.
#[derive(Debug, Default)]
pub struct ValueStack {
    providers: ProviderStack,
    reg_alloc: RegisterAlloc,
}

impl ValueStack {
    /// Resets the [`ValueStack`].
    pub fn reset(&mut self) {
        self.providers.reset();
        self.reg_alloc.reset();
    }

    /// Returns the number of registers allocated by the [`RegisterAlloc`].
    pub fn len_registers(&self) -> u16 {
        self.reg_alloc.len_registers()
    }

    /// Registers an `amount` of function inputs or local variables.
    ///
    /// # Errors
    ///
    /// If too many registers have been registered.
    ///
    /// # Panics
    ///
    /// If the current [`AllocPhase`] is not [`AllocPhase::Init`].
    pub fn register_locals(&mut self, amount: u32) -> Result<(), TranslationError> {
        self.reg_alloc.register_locals(amount)
    }

    /// Pushes a [`Register`] to the [`ValueStack`] referring to a function parameter or local variable.
    pub fn push_local(&mut self, local_index: u32) -> Result<Register, TranslationError> {
        let index = u16::try_from(local_index)
            .ok()
            .filter(|&value| value <= self.reg_alloc.len_locals())
            .ok_or_else(|| TranslationError::new(TranslationErrorInner::RegisterOutOfBounds))?;
        let reg = Register::from_u16(index);
        self.providers.push_local(reg);
        Ok(reg)
    }

    /// Pushes a dynamically allocated [`Register`] to the [`ValueStack`].
    pub fn push_dynamic(&mut self) -> Result<Register, TranslationError> {
        let reg = self.reg_alloc.push_dynamic()?;
        self.providers.push_dynamic(reg);
        Ok(reg)
    }

    /// Pops the top-most [`Provider`] from the [`ValueStack`].
    pub fn pop(&mut self) -> Provider {
        match self.providers.pop() {
            TaggedProvider::Local(reg) => Provider::Register(reg),
            TaggedProvider::Dynamic(reg) => {
                self.reg_alloc.pop_dynamic();
                Provider::Register(reg)
            }
            TaggedProvider::Storage(reg) => {
                self.reg_alloc.pop_storage();
                Provider::Register(reg)
            }
            TaggedProvider::Const(value) => Provider::Const(value),
        }
    }

    /// Registers the [`Instr`] user for [`Register`] if `reg` is allocated in storage space.
    ///
    /// # Note
    ///
    /// This is required in order to update [`Register`] indices of storage space
    /// allocated registers after register allocation is finished.
    pub fn register_user(&mut self, reg: Register, user: Instr) {
        self.reg_alloc.register_user(reg, user)
    }

    /// Defragments the allocated registers space.
    ///
    /// # Note
    ///
    /// This is needed because dynamically allocated registers and storage space allocated
    /// registers do not have consecutive index spaces for technical reasons. This is why we
    /// store the definition site and users of storage space allocated registers so that we
    /// can defrag exactly those registers and make the allocated register space compact.
    pub fn defrag(&mut self, state: &mut impl DefragRegister) {
        self.reg_alloc.defrag(state)
    }
}
