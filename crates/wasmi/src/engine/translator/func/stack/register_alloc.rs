use super::{TaggedProvider, TypedProvider};
use crate::{
    engine::TranslationError,
    ir::{Reg, RegSpan},
    Error,
};
use alloc::collections::BTreeSet;
use core::{
    cmp::{max, min},
    num::NonZeroUsize,
};
use multi_stash::{Key, Key as StashKey, MultiStash};

#[cfg(doc)]
use crate::engine::translator::func::InstrEncoder;

/// The register allocator using during translation.
///
/// # Note
///
/// This has two phases:
///
/// 1. `init`:
///    The initialization phase registers all function inputs
///    and local variables during parsing. After parsing all
///    function inputs and local variables the `alloc` phase
///    is started.
/// 2. `alloc`:
///    The allocation phase drives the allocation of dynamically
///    used registers. These are registers that are not function
///    inputs or registered local variables that are implicitly
///    used during instruction execution, for example to hold
///    and accumulate computation results temporarily.
/// 3. `defrag`:
///    The allocation phase has finished and the register allocator
///    can now defragment allocated register space to form a consecutive
///    block of registers in use by the function.
///
/// The stack of registers is always ordered in this way:
///
/// | `inputs` | `locals` | `dynamics` | `preservation` |
///
/// Where
///
/// - `inputs`: function inputs
/// - `locals`: function local variables
/// - `dynamics:` dynamically allocated registers
/// - `preservations:` registers holding state for later use
///
/// The dynamically allocated registers grow upwards
/// starting with the lowest index possible whereas the
/// preservation registers are growing downwards starting with
/// the largest index (`u16::MAX`).
/// If both meet we officially ran out of registers to allocate.
///
/// After allocation the preservation registers are normalized and
/// simply appended to the dynamically registers to form a
/// consecutive block of registers for the function.
#[derive(Debug, Default)]
pub struct RegisterAlloc {
    /// The preservation stack.
    preservations: MultiStash<()>,
    /// Keys that might have been fully removed from the `preservations` stack.
    ///
    /// When popping a preserved register we store its key into this set.
    /// The next time allocating a preserved register we first check if any
    /// of the preserved register allocations in this set are now fully unused
    /// and then remove them. We achieve this by having a starting amount of 2.
    ///
    /// This allows to extend the lifetimes of preserved registers so that we
    /// can re-push them in case we still need them until the next allocation.
    removed_preserved: BTreeSet<Key>,
    /// The current phase of the register allocation procedure.
    phase: AllocPhase,
    /// The combined number of registered function inputs and local variables.
    len_locals: u16,
    /// The index for the next dynamically allocated register.
    next_dynamic: i16,
    /// The maximum index registered for a dynamically allocated register.
    max_dynamic: i16,
    /// The minimum index registered for a preservation allocated register.
    min_preserve: i16,
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
    /// can defragment registers allocated for preservation purposes
    /// to form a consecutive register stack.
    ///
    /// # Note
    ///
    /// This state entirely disallows allocating registers, function
    /// inputs or local variables.
    Defrag,
}

/// The [`RegisterSpace`] of a [`Reg`].
#[derive(Debug, Copy, Clone)]
pub enum RegisterSpace {
    /// Function local constant values are assigned to this [`RegisterSpace`].
    Const,
    /// Function parameters and local variables are assigned to this [`RegisterSpace`].
    Local,
    /// Dynamically allocated parameters are assigned to this [`RegisterSpace`].
    Dynamic,
    /// Preserved local variables are assigned to this [`RegisterSpace`].
    Preserve,
}

impl RegisterAlloc {
    /// The maximum amount of local variables (and function parameters) a function may define.
    const MAX_LEN_LOCALS: u16 = i16::MAX as u16 - 1;

    /// The initial preservation register index.
    const INITIAL_PRESERVATION_INDEX: i16 = i16::MAX - 1;

    /// Resets the [`RegisterAlloc`] to start compiling a new function.
    pub fn reset(&mut self) {
        self.preservations.clear();
        self.phase = AllocPhase::Init;
        self.len_locals = 0;
        self.next_dynamic = 0;
        self.max_dynamic = 0;
        self.min_preserve = Self::INITIAL_PRESERVATION_INDEX;
        self.defrag_offset = 0;
    }

    /// Adjusts the [`RegisterAlloc`] for the popped [`TaggedProvider`] and returns a [`TypedProvider`].
    pub fn pop_provider(&mut self, provider: TaggedProvider) -> TypedProvider {
        match provider {
            TaggedProvider::Local(reg) => TypedProvider::Register(reg),
            TaggedProvider::Dynamic(reg) => {
                self.pop_dynamic();
                TypedProvider::Register(reg)
            }
            TaggedProvider::Preserved(reg) => {
                self.pop_preserved(reg);
                TypedProvider::Register(reg)
            }
            TaggedProvider::ConstLocal(reg) => TypedProvider::Register(reg),
            TaggedProvider::ConstValue(value) => TypedProvider::Const(value),
        }
    }

    /// Returns the [`RegisterSpace`] for the given [`Reg`].
    pub fn register_space(&self, register: Reg) -> RegisterSpace {
        if register.is_const() {
            return RegisterSpace::Const;
        }
        if self.is_local(register) {
            return RegisterSpace::Local;
        }
        if self.is_preserved(register) {
            return RegisterSpace::Preserve;
        }
        RegisterSpace::Dynamic
    }

    /// Returns thenumber of registers allocated as function parameters or locals.
    pub fn len_locals(&self) -> u16 {
        self.len_locals
    }

    /// Returns the minimum index of any dynamically allocated [`Reg`].
    fn min_dynamic(&self) -> i16 {
        self.len_locals() as i16
    }

    /// Returns the number of registers allocated by the [`RegisterAlloc`].
    pub fn len_registers(&self) -> u16 {
        Self::MAX_LEN_LOCALS - self.max_dynamic.abs_diff(self.min_preserve)
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
    pub fn register_locals(&mut self, amount: u32) -> Result<(), Error> {
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
        self.len_locals = bump_locals(self.len_locals, amount)
            .ok_or_else(|| Error::from(TranslationError::AllocatedTooManyRegisters))?;
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
    /// However, it is then possible to push and pop dynamic and preserved registers to the stack.
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

    /// Allocates a new [`Reg`] on the dynamic allocation stack and returns it.
    ///
    /// # Errors
    ///
    /// If too many registers have been registered.
    ///
    /// # Panics
    ///
    /// If the current [`AllocPhase`] is not [`AllocPhase::Alloc`].
    pub fn push_dynamic(&mut self) -> Result<Reg, Error> {
        self.assert_alloc_phase();
        if self.next_dynamic == self.min_preserve {
            return Err(Error::from(TranslationError::AllocatedTooManyRegisters));
        }
        let reg = Reg::from(self.next_dynamic);
        self.next_dynamic += 1;
        self.max_dynamic = max(self.max_dynamic, self.next_dynamic);
        Ok(reg)
    }

    /// Allocates `n` new [`Reg`] on the dynamic allocation stack and returns them.
    ///
    /// # Errors
    ///
    /// If too many registers have been registered.
    ///
    /// # Panics
    ///
    /// If the current [`AllocPhase`] is not [`AllocPhase::Alloc`].
    pub fn push_dynamic_n(&mut self, n: usize) -> Result<RegSpan, Error> {
        fn next_dynamic_n(this: &mut RegisterAlloc, n: usize) -> Option<RegSpan> {
            let n = i16::try_from(n).ok()?;
            let next_dynamic = this.next_dynamic.checked_add(n)?;
            if next_dynamic >= this.min_preserve {
                return None;
            }
            let register = RegSpan::new(Reg::from(this.next_dynamic));
            this.next_dynamic += n;
            this.max_dynamic = max(this.max_dynamic, this.next_dynamic);
            Some(register)
        }
        self.assert_alloc_phase();
        next_dynamic_n(self, n)
            .ok_or_else(|| Error::from(TranslationError::AllocatedTooManyRegisters))
    }

    /// Pops the top-most dynamically allocated [`Reg`] from the allocation stack.
    ///
    /// # Panics
    ///
    /// - If the dynamic register allocation stack is empty.
    /// - If the current [`AllocPhase`] is not [`AllocPhase::Alloc`].
    fn pop_dynamic(&mut self) {
        self.assert_alloc_phase();
        assert_ne!(
            self.next_dynamic,
            self.min_dynamic(),
            "dynamic register allocation stack is empty"
        );
        self.next_dynamic -= 1;
    }

    /// Pops the `n` top-most dynamically allocated [`Reg`] from the allocation stack.
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

    /// Allocates a new [`Reg`] on the preservation stack and returns it.
    ///
    /// # Note
    ///
    /// Registers allocated to the preservation space generally need
    /// to be readjusted later on in order to have a consecutive register space.
    ///
    /// # Errors
    ///
    /// If too many registers have been registered.
    ///
    /// # Panics
    ///
    /// If the current [`AllocPhase`] is not [`AllocPhase::Alloc`].
    pub fn push_preserved(&mut self) -> Result<Reg, Error> {
        const NZ_TWO: NonZeroUsize = match NonZeroUsize::new(2) {
            Some(value) => value,
            None => unreachable!(),
        };
        self.assert_alloc_phase();
        // Now we can clear the removed preserved registers.
        self.removed_preserved.clear();
        let key = self.preservations.put(NZ_TWO, ());
        let reg = Self::key2reg(key);
        self.update_min_preserved(reg.prev())?;
        Ok(reg)
    }

    /// Frees all preservation slots that are flagged for removal.
    ///
    /// This is important to allow them for reuse for future preservations.
    pub fn gc_preservations(&mut self) {
        self.assert_alloc_phase();
        if self.removed_preserved.is_empty() {
            return;
        }
        for &key in &self.removed_preserved {
            let entry = self.preservations.get(key);
            debug_assert!(
                !matches!(entry, Some((0, _))),
                "found preserved register allocation entry with invalid 0 amount"
            );
            if let Some((1, _)) = entry {
                // Case: we only have one preservation left which
                //       indicates that all preserved registers have
                //       been used, thus we can remove this entry
                //       which makes it available for allocation again.
                self.preservations.take_all(key);
            }
        }
    }

    /// Bumps the [`Reg`] quantity on the preservation stack by one.
    ///
    /// # Panics
    ///
    /// If `register` is not a preservation [`Reg`].
    pub fn bump_preserved(&mut self, register: Reg) {
        debug_assert!(matches!(
            self.register_space(register),
            RegisterSpace::Preserve
        ));
        let key = Self::reg2key(register);
        let old_amount = self.preservations.bump(key, 1);
        debug_assert!(
            // Note: We check that the returned value is `Some` to guard
            //       against unexpected vacant entries at the `register` slot.
            old_amount.is_some()
        );
    }

    /// Pops the [`Reg`] from the preservation stack.
    ///
    /// # Panics
    ///
    /// - If the dynamic register allocation stack is empty.
    /// - If the current [`AllocPhase`] is not [`AllocPhase::Alloc`].
    fn pop_preserved(&mut self, register: Reg) {
        self.assert_alloc_phase();
        let key = Self::reg2key(register);
        self.removed_preserved.insert(key);
        self.preservations
            .take_one(key)
            .unwrap_or_else(|| panic!("missing preservation slot for {register:?}"));
    }

    /// Updates the minimum preservation [`Reg`] index if needed.
    fn update_min_preserved(&mut self, register: Reg) -> Result<(), Error> {
        self.min_preserve = min(self.min_preserve, i16::from(register));
        if self.next_dynamic == self.min_preserve {
            return Err(Error::from(TranslationError::AllocatedTooManyRegisters));
        }
        Ok(())
    }

    /// Converts a preservation [`Reg`] into a [`StashKey`].
    fn reg2key(register: Reg) -> StashKey {
        let reg_index = Self::INITIAL_PRESERVATION_INDEX - i16::from(register);
        let key_index = usize::try_from(reg_index).unwrap_or_else(|error| {
            panic!("reg_index ({reg_index}) must be convertible to usize: {error}")
        });
        StashKey::from(key_index)
    }

    /// Converts a [`StashKey`] into a preservation [`Reg`].
    fn key2reg(key: StashKey) -> Reg {
        let key_index = usize::from(key);
        let reg_index = Self::INITIAL_PRESERVATION_INDEX
            - i16::try_from(key_index).unwrap_or_else(|error| {
                panic!(
                    "key_index ({key_index}) must be convertible to positive i16 integer: {error}"
                )
            });
        Reg::from(reg_index)
    }

    /// Returns `true` if the [`Reg`] is allocated in the [`RegisterSpace::Local`].
    pub fn is_local(&self, reg: Reg) -> bool {
        !reg.is_const() && i16::from(reg) < self.min_dynamic()
    }

    /// Returns `true` if the [`Reg`] is allocated in the [`RegisterSpace::Preserve`].
    fn is_preserved(&self, reg: Reg) -> bool {
        self.min_preserve < i16::from(reg)
    }

    /// Finalizes register allocation and allows to defragment the register space.
    pub fn finalize_alloc(&mut self) {
        assert!(matches!(self.phase, AllocPhase::Alloc));
        self.phase = AllocPhase::Defrag;
        self.defrag_offset = (self.min_preserve - self.max_dynamic).saturating_add(1);
    }

    /// Returns the defragmented [`Reg`].
    pub fn defrag_register(&self, register: Reg) -> Reg {
        assert!(matches!(self.phase, AllocPhase::Defrag));
        if !self.is_preserved(register) {
            // Only registers allocated to the preservation space need defragmentation.
            return register;
        }
        Reg::from(i16::from(register) - self.defrag_offset)
    }

    /// Increase preservation [`Reg`] usage.
    ///
    /// # Note
    ///
    /// - This is mainly used to extend the lifetime of `else` providers on the stack.
    /// - This does nothing if `register` is not a preservation [`Reg`].
    pub fn inc_register_usage(&mut self, register: Reg) {
        if !self.is_preserved(register) {
            return;
        }
        self.bump_preserved(register)
    }

    /// Decrease preservation [`Reg`] usage.
    ///
    /// # Note
    ///
    /// - This is mainly used to shorten the lifetime of `else` providers on the stack.
    /// - This does nothing if `register` is not a preservation [`Reg`].
    pub fn dec_register_usage(&mut self, register: Reg) {
        if !self.is_preserved(register) {
            return;
        }
        self.pop_preserved(register)
    }
}
