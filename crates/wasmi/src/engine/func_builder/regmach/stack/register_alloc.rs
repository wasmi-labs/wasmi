use super::{Provider, TaggedProvider};
use crate::engine::{
    bytecode2::Register,
    func_builder::{Instr, TranslationErrorInner},
    TranslationError,
};
use alloc::collections::btree_set::BTreeSet;
use core::cmp::{max, min};

#[cfg(doc)]
use crate::engine::func_builder::regmach::InstrEncoder;

/// The register allocator using during translation.
///
/// # Note
///
/// This has two phases:
///
/// 1. `init`:
///     The initialization phase registers all function inputs
///     and local variables during parsing. After parsing all
///     function inputs and local variables the `alloc` phase
///     is started.
/// 2. `alloc`:
///     The allocation phase drives the allocation of dynamically
///     used registers. These are registers that are not function
///     inputs or registered local variables that are implicitely
///     used during instruction execution, for example to hold
///     and accumulate computation results temporarily.
/// 3. `defrag`:
///     The allocation phase has finished and the register allocator
///     can now defragment allocated register space to form a consecutive
///     block of registers in use by the function.
///
/// The stack of registers is always ordered in this way:
///
/// | `inputs` | `locals` | `dynamics` | `storage` |
///
/// Where
///
/// - `inputs`: function inputs
/// - `locals`: function local variables
/// - `dynamics:` dynamically allocated registers
/// - `storage:` registers holding state for later use
///
/// The dynamically allocated registers grow upwards
/// starting with the lowest index possible whereas the
/// storage registers are growing downwards starting with
/// the largest index (`u16::MAX`).
/// If both meet we officially ran out of registers to allocate.
///
/// After allocation the storage registers are normalized and
/// simply appended to the dynamically registers to form a
/// consecutive block of registers for the function.
#[derive(Debug, Default)]
pub struct RegisterAlloc {
    /// The current phase of the register allocation procedure.
    phase: AllocPhase,
    /// The combined number of registered function inputs and local variables.
    len_locals: u16,
    /// The index for the next dynamically allocated register.
    next_dynamic: u16,
    /// The maximum index registered for a dynamically allocated register.
    max_dynamic: u16,
    /// The index for the next register allocated to the storage.
    next_storage: u16,
    /// The minimum index registered for a storage allocated register.
    min_storage: u16,
    /// Storage register users and definition sites.
    storage_users: BTreeSet<RegisterUser>,
}

/// A pair of [`Register`] and definition site or user of the [`Register`].
///
/// # Note
///
/// This is only required for storage space allocated registers.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegisterUser {
    /// The storage space allocated register that will need index adjustment.
    register: Register,
    /// The user or definition site of the storage space allocated [`Register`].
    user: Instr,
}

impl RegisterUser {
    /// Creates a new [`RegisterUser`] pair.
    pub fn new(register: Register, user: Instr) -> Self {
        Self { register, user }
    }

    /// Returns the [`Register`].
    pub fn reg(&self) -> Register {
        self.register
    }

    /// Returns the user or definition site of the [`Register`].
    pub fn user(&self) -> Instr {
        self.user
    }
}

/// The phase of the [`RegisterAlloc`].
#[derive(Debug, Default, Copy, Clone)]
enum AllocPhase {
    /// The [`RegisterAlloc`] is in its initialization phase.
    ///
    /// # Note
    ///
    /// This disallows allocating registers but allows registering local variables.
    #[default]
    Init,
    /// The [`RegisterAlloc`] is in its allocation phase.
    ///
    /// # Note
    ///
    /// This disallows registering new local variables to the [`RegisterAlloc`].
    Alloc,
    /// The [`RegisterAlloc`] finished allocation and now
    /// can defragment registers allocated for storage purposes
    /// to form a consecutive register stack.
    ///
    /// # Note
    ///
    /// This state entirely disallows allocating registers, function
    /// inputs or local variables.
    Defrag,
}

impl RegisterAlloc {
    /// Resets the [`RegisterAlloc`] to start compiling a new function.
    pub fn reset(&mut self) {
        self.phase = AllocPhase::Init;
        self.len_locals = 0;
        self.next_dynamic = 0;
        self.max_dynamic = 0;
        self.next_storage = u16::MAX;
        self.min_storage = u16::MAX;
    }

    /// Adjusts the [`RegisterAlloc`] for the popped [`TaggedProvider`] and returns a [`Provider`].
    pub fn pop_provider(&mut self, provider: TaggedProvider) -> Provider {
        match provider {
            TaggedProvider::Local(reg) => Provider::Register(reg),
            TaggedProvider::Dynamic(reg) => {
                self.pop_dynamic();
                Provider::Register(reg)
            }
            TaggedProvider::Storage(reg) => {
                self.pop_storage();
                Provider::Register(reg)
            }
            TaggedProvider::Const(value) => Provider::Const(value),
        }
    }

    /// Returns thenumber of registers allocated as function parameters or locals.
    pub fn len_locals(&self) -> u16 {
        self.len_locals
    }

    /// Returns the number of registers allocated by the [`RegisterAlloc`].
    pub fn len_registers(&self) -> u16 {
        let len_dynamic = self.max_dynamic - self.len_locals;
        let len_storage = u16::MAX - self.min_storage;
        self.len_locals + len_dynamic + len_storage
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
        /// Bumps `len_locals` by `amount` if possible.
        fn bump_locals(len_locals: u16, amount: u32) -> Option<u16> {
            let amount = u16::try_from(amount).ok()?;
            len_locals.checked_add(amount)
        }
        assert!(matches!(self.phase, AllocPhase::Init));
        self.len_locals = bump_locals(self.len_locals, amount).ok_or_else(|| {
            TranslationError::new(TranslationErrorInner::AllocatedTooManyRegisters)
        })?;
        self.next_dynamic = self.len_locals;
        self.max_dynamic = self.len_locals;
        Ok(())
    }

    /// Finishes [`AllocPhase::Init`].
    ///
    /// # Note
    ///
    /// After this operation no local variable can be registered anymore.
    /// However, it is then possible to push and pop dynamic and storage registers to the stack.
    pub fn finish_register_locals(&mut self) {
        assert!(matches!(self.phase, AllocPhase::Init));
        self.phase = AllocPhase::Alloc;
    }

    /// Asserts that the [`RegisterAlloc`] is in [`AllocPhase::Init`] or [`AllocPhase::Alloc`].
    ///
    /// Makes sure the [`RegisterAlloc`] is in [`AllocPhase::Alloc`] after this call.
    fn assert_alloc_phase(&mut self) {
        assert!(matches!(self.phase, AllocPhase::Alloc));
    }

    /// Allocates a new [`Register`] on the dynamic allocation stack and returns it.
    ///
    /// # Errors
    ///
    /// If too many registers have been registered.
    ///
    /// # Panics
    ///
    /// If the current [`AllocPhase`] is not [`AllocPhase::Alloc`].
    pub fn push_dynamic(&mut self) -> Result<Register, TranslationError> {
        self.assert_alloc_phase();
        if self.next_dynamic == self.next_storage {
            return Err(TranslationError::new(
                TranslationErrorInner::AllocatedTooManyRegisters,
            ));
        }
        let reg = Register::from_u16(self.next_dynamic);
        self.next_dynamic += 1;
        self.max_dynamic = max(self.max_dynamic, self.next_dynamic);
        Ok(reg)
    }

    /// Allocates a new [`Register`] on the storage allocation stack and returns it.
    ///
    /// # Note
    ///
    /// - Registers allocated to the storage allocation space generally need
    ///   to be readjusted later on in order to have a consecutive register space.
    /// - Requires a definition site (`def_site`) to register the [`Instr`] where
    ///   the register is defined in order to adjust the register index easily later.
    ///
    /// # Errors
    ///
    /// If too many registers have been registered.
    ///
    /// # Panics
    ///
    /// If the current [`AllocPhase`] is not [`AllocPhase::Alloc`].
    pub fn push_storage(&mut self, def_site: Instr) -> Result<Register, TranslationError> {
        self.assert_alloc_phase();
        if self.next_dynamic == self.next_storage {
            return Err(TranslationError::new(
                TranslationErrorInner::AllocatedTooManyRegisters,
            ));
        }
        let reg = Register::from_u16(self.next_storage);
        self.storage_users.insert(RegisterUser::new(reg, def_site));
        self.next_storage -= 1;
        self.min_storage = min(self.min_storage, self.next_storage);
        Ok(reg)
    }

    /// Pops the top-most dynamically allocated [`Register`] from the allocation stack.
    ///
    /// # Panics
    ///
    /// - If the dynamic register allocation stack is empty.
    /// - If the current [`AllocPhase`] is not [`AllocPhase::Alloc`].
    pub fn pop_dynamic(&mut self) {
        self.assert_alloc_phase();
        assert_ne!(
            self.next_dynamic, self.len_locals,
            "dynamic register allocation stack is empty"
        );
        self.next_dynamic -= 1;
    }

    /// Pops the top-most dynamically allocated [`Register`] from the allocation stack.
    ///
    /// # Panics
    ///
    /// - If the dynamic register allocation stack is empty.
    /// - If the current [`AllocPhase`] is not [`AllocPhase::Alloc`].
    pub fn pop_storage(&mut self) {
        self.assert_alloc_phase();
        assert_ne!(
            self.next_storage,
            u16::MAX,
            "storage register allocation stack is empty"
        );
        self.next_storage += 1;
    }

    /// Registers the [`Instr`] user for [`Register`] if `reg` is allocated in storage space.
    ///
    /// # Note
    ///
    /// This is required in order to update [`Register`] indices of storage space
    /// allocated registers after register allocation is finished.
    pub fn register_user(&mut self, reg: Register, user: Instr) {
        if self.is_storage(reg) {
            self.storage_users.insert(RegisterUser::new(reg, user));
        }
    }

    /// Returns `true` if the [`Register`] is allocated in the dynamic register space.
    pub fn is_dynamic(&self, reg: Register) -> bool {
        self.len_locals <= reg.to_u16() && reg.to_u16() < self.max_dynamic
    }

    /// Returns `true` if the [`Register`] is allocated in the storage register space.
    pub fn is_storage(&self, reg: Register) -> bool {
        self.min_storage < reg.to_u16()
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
        assert!(matches!(self.phase, AllocPhase::Alloc));
        self.phase = AllocPhase::Defrag;
        if self.next_dynamic == self.next_storage {
            // This is a very special edge case in which all registers are
            // already in use and we cannot really adjust anything anymore.
            return;
        }
        self.next_storage = self.max_dynamic;
        for user in &self.storage_users {
            let reg = user.reg();
            let instr = user.user();
            let new_reg = Register::from_u16(self.next_storage);
            state.defrag_register(instr, reg, new_reg);
            self.next_storage += 1;
        }
    }
}

/// Allows to defragment the index of registers of instructions.
///
/// # Note
///
/// This is usually implemented by the [`InstrEncoder`]
/// so that the [`InstrEncoder`] can be informed by the [`RegisterAlloc`] about
/// storage space allocated registers that need to be defragmented.
pub trait DefragRegister {
    /// Adjusts [`Register`] `reg` of [`Instr`] `user` to [`Register`] `new_reg`.
    fn defrag_register(&mut self, user: Instr, reg: Register, new_reg: Register);
}
