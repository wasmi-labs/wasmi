use super::{TaggedProvider, TypedProvider};
use crate::engine::{
    func_builder::TranslationErrorInner,
    regmach::bytecode::{Register, RegisterSpan},
    TranslationError,
};
use core::{
    cmp::{max, min},
    num::NonZeroUsize,
};
use multi_stash::{Key as StashKey, MultiStash};

#[cfg(doc)]
use crate::engine::regmach::translator::InstrEncoder;

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
///     inputs or registered local variables that are implicitly
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
    /// The preservation stack.
    preservations: MultiStash<()>,
    /// The current phase of the register allocation procedure.
    phase: AllocPhase,
    /// The combined number of registered function inputs and local variables.
    len_locals: u16,
    /// The index for the next dynamically allocated register.
    next_dynamic: i16,
    /// The maximum index registered for a dynamically allocated register.
    max_dynamic: i16,
    /// The minimum index registered for a storage allocated register.
    min_storage: i16,
    /// The offset for the defragmentation register index.
    defrag_offset: i16,
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

/// The [`RegisterSpace`] of a [`Register`].
#[derive(Debug, Copy, Clone)]
pub enum RegisterSpace {
    /// Function local constant values are assigned to this [`RegisterSpace`].
    Const,
    /// Function parameters and local variables are assigned to this [`RegisterSpace`].
    Local,
    /// Dynamically allocated parameters are assigned to this [`RegisterSpace`].
    Dynamic,
    /// Preserved local variables are assigned to this [`RegisterSpace`].
    Storage,
}

impl RegisterAlloc {
    /// The maximum amount of local variables (and function parameters) a function may define.
    const MAX_LEN_LOCALS: u16 = i16::MAX as u16;

    /// Resets the [`RegisterAlloc`] to start compiling a new function.
    pub fn reset(&mut self) {
        self.phase = AllocPhase::Init;
        self.len_locals = 0;
        self.next_dynamic = 0;
        self.max_dynamic = 0;
        self.min_storage = i16::MAX;
    }

    /// Adjusts the [`RegisterAlloc`] for the popped [`TaggedProvider`] and returns a [`TypedProvider`].
    pub fn pop_provider(&mut self, provider: TaggedProvider) -> TypedProvider {
        match provider {
            TaggedProvider::Local(reg) => TypedProvider::Register(reg),
            TaggedProvider::Dynamic(reg) => {
                self.pop_dynamic();
                TypedProvider::Register(reg)
            }
            TaggedProvider::Storage(reg) => {
                self.pop_storage(reg);
                TypedProvider::Register(reg)
            }
            TaggedProvider::ConstLocal(reg) => TypedProvider::Register(reg),
            TaggedProvider::ConstValue(value) => TypedProvider::Const(value),
        }
    }

    /// Returns the [`RegisterSpace`] for the given [`Register`].
    pub fn register_space(&self, register: Register) -> RegisterSpace {
        if register.is_const() {
            return RegisterSpace::Const;
        }
        if self.is_local(register) {
            return RegisterSpace::Local;
        }
        if self.is_storage(register) {
            return RegisterSpace::Storage;
        }
        RegisterSpace::Dynamic
    }

    /// Returns thenumber of registers allocated as function parameters or locals.
    pub fn len_locals(&self) -> u16 {
        self.len_locals
    }

    /// Returns the minimum index of any dynamically allocated [`Register`].
    pub fn min_dynamic(&self) -> i16 {
        self.len_locals() as i16
    }

    /// Returns the number of registers allocated by the [`RegisterAlloc`].
    pub fn len_registers(&self) -> u16 {
        (i16::MAX as u16) - self.max_dynamic.abs_diff(self.min_storage)
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
            let new_len = len_locals.checked_add(amount)?;
            if new_len >= RegisterAlloc::MAX_LEN_LOCALS {
                return None;
            }
            Some(new_len)
        }
        assert!(matches!(self.phase, AllocPhase::Init));
        self.len_locals = bump_locals(self.len_locals, amount).ok_or_else(|| {
            TranslationError::new(TranslationErrorInner::AllocatedTooManyRegisters)
        })?;
        // We can convert `len_locals` to `i16` because it is always without bounds of `0..i16::MAX`.
        self.next_dynamic = self.len_locals as i16;
        self.max_dynamic = self.len_locals as i16;
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
    fn assert_alloc_phase(&self) {
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
        if self.next_dynamic == self.min_storage {
            return Err(TranslationError::new(
                TranslationErrorInner::AllocatedTooManyRegisters,
            ));
        }
        let reg = Register::from_i16(self.next_dynamic);
        self.next_dynamic += 1;
        self.max_dynamic = max(self.max_dynamic, self.next_dynamic);
        Ok(reg)
    }

    /// Allocates `n` new [`Register`] on the dynamic allocation stack and returns them.
    ///
    /// # Errors
    ///
    /// If too many registers have been registered.
    ///
    /// # Panics
    ///
    /// If the current [`AllocPhase`] is not [`AllocPhase::Alloc`].
    pub fn push_dynamic_n(&mut self, n: usize) -> Result<RegisterSpan, TranslationError> {
        fn next_dynamic_n(this: &mut RegisterAlloc, n: usize) -> Option<RegisterSpan> {
            let n = i16::try_from(n).ok()?;
            let next_dynamic = this.next_dynamic.checked_add(n)?;
            if next_dynamic >= this.min_storage {
                return None;
            }
            let register = RegisterSpan::new(Register::from_i16(this.next_dynamic));
            this.next_dynamic += n;
            this.max_dynamic = max(this.max_dynamic, this.next_dynamic);
            Some(register)
        }
        self.assert_alloc_phase();
        next_dynamic_n(self, n)
            .ok_or_else(|| TranslationError::new(TranslationErrorInner::AllocatedTooManyRegisters))
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
            self.next_dynamic,
            self.min_dynamic(),
            "dynamic register allocation stack is empty"
        );
        self.next_dynamic -= 1;
    }

    /// Pops the `n` top-most dynamically allocated [`Register`] from the allocation stack.
    ///
    /// # Panics
    ///
    /// - If the dynamic register allocation stack is empty.
    /// - If the current [`AllocPhase`] is not [`AllocPhase::Alloc`].
    pub fn pop_dynamic_n(&mut self, n: usize) {
        fn pop_impl(this: &mut RegisterAlloc, n: usize) -> Option<()> {
            let n = i16::try_from(n).ok()?;
            let new_next_dynamic = this.next_dynamic.checked_sub(n)?;
            if new_next_dynamic < this.min_dynamic() {
                return None;
            }
            this.next_dynamic = new_next_dynamic;
            Some(())
        }
        self.assert_alloc_phase();
        pop_impl(self, n).expect("dynamic register underflow")
    }

    /// Allocates a new [`Register`] on the storage allocation stack and returns it.
    ///
    /// # Note
    ///
    /// Registers allocated to the storage allocation space generally need
    /// to be readjusted later on in order to have a consecutive register space.
    ///
    /// # Errors
    ///
    /// If too many registers have been registered.
    ///
    /// # Panics
    ///
    /// If the current [`AllocPhase`] is not [`AllocPhase::Alloc`].
    pub fn push_storage(&mut self) -> Result<Register, TranslationError> {
        self.assert_alloc_phase();
        let key = self.preservations.put(NonZeroUsize::new(1).unwrap(), ());
        let reg = Self::key2reg(key);
        self.update_min_storage(reg.prev())?;
        Ok(reg)
    }

    /// Bumps the [`Register`] quantity on the preservation stack by one.
    ///
    /// # Panics
    ///
    /// If `register` is not a preservation [`Register`].
    pub fn bump_storage(&mut self, register: Register) {
        let key = Self::reg2key(register);
        self.preservations.bump(key, 1);
    }

    /// Pops the [`Register`] from the preservation stack.
    ///
    /// # Panics
    ///
    /// - If the dynamic register allocation stack is empty.
    /// - If the current [`AllocPhase`] is not [`AllocPhase::Alloc`].
    pub fn pop_storage(&mut self, register: Register) {
        self.assert_alloc_phase();
        let key = Self::reg2key(register);
        self.preservations
            .take_one(key)
            .expect("missing preservation slot for {register:?}");
    }

    /// Updates the minimum preservation [`Register`] index if needed.
    fn update_min_storage(&mut self, register: Register) -> Result<(), TranslationError> {
        self.min_storage = min(self.min_storage, register.to_i16());
        if self.next_dynamic == self.min_storage {
            return Err(TranslationError::new(
                TranslationErrorInner::AllocatedTooManyRegisters,
            ));
        }
        Ok(())
    }

    /// Converts a preservation [`Register`] into a [`StashKey`].
    fn reg2key(register: Register) -> StashKey {
        let reg_index = i16::MAX - register.to_i16();
        let key_index = usize::try_from(reg_index).unwrap_or_else(|error| {
            panic!("reg_index ({reg_index}) must be convertible to usize: {error}")
        });
        StashKey::from(key_index)
    }

    /// Converts a [`StashKey`] into a preservation [`Register`].
    fn key2reg(key: StashKey) -> Register {
        let key_index = usize::from(key);
        let reg_index = i16::MAX
            - i16::try_from(key_index).unwrap_or_else(|error| {
                panic!(
                    "key_index ({key_index}) must be convertible to positive i16 integer: {error}"
                )
            });
        Register::from_i16(reg_index)
    }

    /// Returns `true` if the [`Register`] is allocated in the [`RegisterSpace::Local`].
    pub fn is_local(&self, reg: Register) -> bool {
        !reg.is_const() && reg.to_i16() < self.min_dynamic()
    }

    /// Returns `true` if the [`Register`] is allocated in the [`RegisterSpace::Storage`].
    pub fn is_storage(&self, reg: Register) -> bool {
        self.min_storage < reg.to_i16()
    }

    /// Finalizes register allocation and allows to defragment the register space.
    pub fn finalize_alloc(&mut self) {
        assert!(matches!(self.phase, AllocPhase::Alloc));
        self.phase = AllocPhase::Defrag;
        self.defrag_offset = (self.min_storage - self.max_dynamic).saturating_add(1);
    }

    /// Returns the defragmented [`Register`].
    pub fn defrag_register(&self, register: Register) -> Register {
        assert!(matches!(self.phase, AllocPhase::Defrag));
        if !self.is_storage(register) {
            // Only registers allocated to the storage-space need defragmentation.
            return register;
        }
        Register::from_i16(register.to_i16() - self.defrag_offset)
    }
}
