use super::instr_encoder::Instr;
use crate::engine::{bytecode2::Register, func_builder::TranslationErrorInner, TranslationError};
use alloc::collections::btree_set::BTreeSet;
use core::cmp::max;

#[cfg(doc)]
use super::instr_encoder::InstrEncoder;

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
    /// The total number of allocated registers.
    len_registers: u16,
    /// The index for the next dynamically allocated register.
    next_dynamic: u16,
    /// The maximum index registered for a dynamically allocated register.
    max_dynamic: u16,
    /// The index for the next register allocated to the storage.
    next_storage: u16,
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
        self.len_registers = 0;
        self.next_dynamic = 0;
        self.next_storage = u16::MAX;
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
        self.len_locals = bump_locals(self.len_locals, amount)
            .ok_or_else(|| TranslationError::new(TranslationErrorInner::TooManyRegistersNeeded))?;
        self.next_dynamic = self.len_locals;
        Ok(())
    }

    /// Asserts that the [`RegisterAlloc`] is in [`AllocPhase::Init`] or [`AllocPhase::Alloc`].
    ///
    /// Makes sure the [`RegisterAlloc`] is in [`AllocPhase::Alloc`] after this call.
    fn assert_alloc_phase(&mut self) {
        assert!(matches!(self.phase, AllocPhase::Init | AllocPhase::Alloc));
        self.phase = AllocPhase::Alloc;
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
                TranslationErrorInner::TooManyRegistersNeeded,
            ));
        }
        let reg = Register::from_u16(self.next_dynamic);
        self.max_dynamic = max(self.max_dynamic, self.next_dynamic);
        self.next_dynamic += 1;
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
                TranslationErrorInner::TooManyRegistersNeeded,
            ));
        }
        let reg = Register::from_u16(self.next_storage);
        self.storage_users.insert(RegisterUser::new(reg, def_site));
        self.next_storage -= 1;
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
    pub fn pop_storage(&mut self, user: Instr) {
        self.assert_alloc_phase();
        assert_ne!(
            self.next_storage,
            u16::MAX,
            "storage register allocation stack is empty"
        );
        let reg = Register::from_u16(self.next_storage);
        self.storage_users.insert(RegisterUser::new(reg, user));
        self.next_storage += 1;
    }

    /// Defragments the allocated registers space.
    ///
    /// # Note
    ///
    /// This is needed because dynamically allocated registers and storage space allocated
    /// registers do not have consecutive index spaces for technical reasons. This is why we
    /// store the definition site and users of storage space allocated registers so that we
    /// can defrag exactly those registers and make the allocated register space compact.
    pub fn defrag(&mut self, state: &mut dyn DefragRegister) {
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
