use super::LocalsRegistry;
use alloc::vec::Drain;
use core::{cmp::max, ops::Range};
use wasmi_core::{UntypedValue, ValueType};

/// A stack of provided inputs for constructed instructions.
#[derive(Debug, Default)]
pub struct Providers {
    pub(super) locals: LocalsRegistry,
    providers: Vec<IrProvider>,
    pub(super) stacks: Stacks,
}

#[derive(Debug, Default)]
pub struct Stacks {
    len_dynamic: usize,
    max_dynamic: usize,
    len_preserved: usize,
    max_preserved: usize,
}

impl Stacks {
    pub fn max_dynamic(&self) -> usize {
        self.max_dynamic
    }

    pub fn max_preserved(&self) -> usize {
        self.max_preserved
    }

    /// Bumps the maximum dynamic register space by the amount of new registers.
    ///
    /// Returns the first register in the continuous slice of registers.
    ///
    /// # Note
    ///
    /// This does not actually allocate registers on the stack but instead
    /// reserves them for later purposes.
    fn bump_max_dynamic(&mut self, amount: usize) -> IrRegister {
        let register = IrRegister::Dynamic(self.len_dynamic);
        self.max_dynamic = max(self.max_dynamic, self.len_dynamic + amount);
        register
    }

    /// Bumps the dynamic register space by the amount of new registers.
    ///
    /// Returns the first register in the continuous slice of registers.
    ///
    /// # Note
    ///
    /// All registers allocated this way are contiguous in their indices.
    fn bump_dynamic(&mut self, amount: usize) -> IrRegister {
        let register = IrRegister::Dynamic(self.len_dynamic);
        self.len_dynamic += amount;
        self.max_dynamic = max(self.max_dynamic, self.len_dynamic);
        register
    }

    fn bump_preserved(&mut self) -> IrRegister {
        let register = IrRegister::Preserved(self.len_preserved);
        self.len_preserved += 1;
        self.max_preserved = max(self.max_preserved, self.len_preserved);
        register
    }

    fn pop(&mut self, popped: &IrProvider) {
        match popped {
            IrProvider::Register(IrRegister::Dynamic(_)) => {
                debug_assert!(self.len_dynamic > 0);
                self.len_dynamic -= 1;
            }
            IrProvider::Register(IrRegister::Preserved(_)) => {
                debug_assert!(self.len_preserved > 0);
                self.len_preserved -= 1;
            }
            _ => (),
        }
    }
}

impl Providers {
    /// Preserves `local.get` values on the provider stack if any.
    ///
    /// Returns `Some` preserved register if any provider had to be preserved.
    pub fn preserve_locals(&mut self, preserve_index: u32) -> Option<IrRegister> {
        let preserve_index = preserve_index as usize;
        let mut preserved: Option<IrRegister> = None;
        for provider in &mut self.providers {
            if let IrProvider::Register(IrRegister::Local(local_index)) = provider {
                if *local_index == preserve_index {
                    let preserved_register = match preserved {
                        Some(register) => register,
                        None => {
                            let new_preserved = self.stacks.bump_preserved();
                            preserved = Some(new_preserved);
                            new_preserved
                        }
                    };
                    *provider = IrProvider::Register(preserved_register);
                }
            }
        }
        preserved
    }

    /// Registers the `amount` of locals with their shared [`ValueType`].
    ///
    /// # Panics
    ///
    /// If too many local variables have been registered.
    pub fn register_locals(&mut self, value_type: ValueType, amount: u32) {
        self.locals.register_locals(value_type, amount)
    }

    pub fn push_local(&mut self, local_id: u32) -> IrRegister {
        assert!(local_id < self.locals.len_registered());
        let register = IrRegister::Local(local_id as usize);
        self.providers.push(register.into());
        register
    }

    pub fn push_dynamic(&mut self) -> IrRegister {
        let register = self.stacks.bump_dynamic(1);
        self.providers.push(register.into());
        register
    }

    pub fn peek_dynamic(&mut self) -> IrRegister {
        self.stacks.bump_max_dynamic(1)
    }

    pub fn peek_dynamic_many(&mut self, amount: usize) -> IrRegisterSlice {
        let len = u16::try_from(amount).unwrap_or_else(|error| {
            panic!("tried to push too many dynamic registers ({amount}): {error}")
        });
        let first = self.stacks.bump_max_dynamic(amount);
        IrRegisterSlice::new(first, len)
    }

    pub fn push_dynamic_many(&mut self, amount: usize) -> IrRegisterSlice {
        let len = u16::try_from(amount).unwrap_or_else(|error| {
            panic!("tried to push too many dynamic registers ({amount}): {error}")
        });
        let first = self.stacks.bump_dynamic(amount);
        let slice = IrRegisterSlice::new(first, len);
        for register in slice {
            self.providers.push(register.into());
        }
        slice
    }

    pub fn push_const<T>(&mut self, value: T) -> IrProvider
    where
        T: Into<UntypedValue>,
    {
        let provider = IrProvider::from(value.into());
        self.providers.push(provider);
        provider
    }

    /// Pops the last provider from the [`Providers`] stack.
    ///
    /// # Panics
    ///
    /// If the stack is empty.
    pub fn pop(&mut self) -> IrProvider {
        let popped = self.providers.pop().expect("unexpected missing provider");
        self.stacks.pop(&popped);
        popped
    }

    pub fn pop2(&mut self) -> (IrProvider, IrProvider) {
        let rhs = self.pop();
        let lhs = self.pop();
        (lhs, rhs)
    }

    pub fn pop3(&mut self) -> (IrProvider, IrProvider, IrProvider) {
        let v2 = self.pop();
        let v1 = self.pop();
        let v0 = self.pop();
        (v0, v1, v2)
    }

    pub fn pop_n(&mut self, depth: usize) -> Drain<IrProvider> {
        let max_index = self.len() as usize;
        debug_assert!(depth <= max_index);
        let min_index = max_index - depth;
        for provider in &self.providers[min_index..] {
            self.stacks.pop(provider);
        }
        self.providers.drain(min_index..)
    }

    pub fn peek2(&self) -> (IrProvider, IrProvider) {
        let len = self.len() as usize;
        assert!(len >= 2);
        let lhs = self.providers[len - 2];
        let rhs = self.providers[len - 1];
        (lhs, rhs)
    }

    /// Returns a shared slice of the last `depth` providers on the stack.
    pub fn peek_n(&self, depth: usize) -> &[IrProvider] {
        let max_index = self.len() as usize;
        debug_assert!(depth <= max_index);
        let min_index = max_index - depth;
        &self.providers[min_index..]
    }

    /// Duplicates the last `depth` providers on the stack.
    ///
    /// Returns a shared slice over the duplicated providers.
    pub fn duplicate_n(&mut self, depth: usize) -> &[IrProvider] {
        let max_index = self.len() as usize;
        debug_assert!(depth <= max_index);
        let min_index = max_index - depth;
        let mut n = 0;
        while n < depth {
            self.providers.push(self.providers[min_index + n]);
            n += 1;
        }
        self.peek_n(depth)
    }

    /// Returns the current length of the emulated [`Providers`].
    pub fn len(&self) -> u32 {
        self.providers.len() as u32
    }

    /// Returns `true` if the emulated [`Providers`] is empty.
    #[allow(dead_code)] // TODO: remove annotation, exists because of clippy
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Shrinks the [`Providers`] to the given height.
    ///
    /// # Panics
    ///
    /// If the [`Providers`] height already is below the height since this
    /// usually indicates a bug in the translation of the Wasm to `wasmi`
    /// bytecode procedures.
    pub fn shrink_to(&mut self, new_height: u32) {
        let current_height = self.len();
        assert!(
            new_height <= current_height,
            "tried to invalidly shrink the value stack of height {} to height {}",
            current_height,
            new_height
        );
        let new_height = usize::try_from(new_height).unwrap_or_else(|error| {
            panic!(
                "could not convert stack height from `u32` to `usize`: {}",
                error
            )
        });
        self.providers
            .drain(new_height..)
            .for_each(|popped| self.stacks.pop(&popped));
    }

    /// Returns the required number of registers for the constructed function.
    pub fn len_required_registers(&self) -> u16 {
        let len_registers = self.stacks.max_dynamic()
            + self.stacks.max_preserved()
            + self.locals.len_registered() as usize;
        len_registers.try_into().unwrap_or_else(|error| {
            panic!("out of bounds function registers required (= {len_registers}): {error}")
        })
    }
}

/// A register provider of any of the existing register space.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IrRegister {
    /// The provided value lives in the local register space.
    ///
    /// # Note
    ///
    /// This includes all function parameters and function local
    /// variables.
    Local(usize),
    /// The provided value lives in the dynamic register space.
    ///
    /// # Note
    ///
    /// Dynamic registers are introduced during translation of Wasm
    /// instructions to store intermediate computation results.
    Dynamic(usize),
    /// The provided value lives in the preserved register space.
    ///
    /// # Note
    ///
    /// Preserved registers are introduced during translation of Wasm
    /// instructions to store results of computations that are going to be
    /// used at a later point.
    ///
    /// The distinction between `Local`, `Dynamic` and `Preserved` is
    /// important since due to construction it is not possible to directly map
    /// a preserved [`IrRegister`] to a [`IrRegister`] at time of construction
    /// but only after a function construction is about to be finalized
    /// and no further instructions are to be inserted.
    ///
    /// At function construction finalization all `Preserved` providers
    /// will be offset by the maximum amount of dynamic registers used
    /// throughout the function. The dynamic register space is the union
    /// of function parameters, function locals and dynamically required
    /// function registers.
    Preserved(usize),
}

impl IrRegister {
    /// Returns `true` if the [`IrRegister`] is in the local register space.
    pub fn is_local(&self) -> bool {
        matches!(self, Self::Local(_))
    }

    /// Returns a new [`IrRegister`] with its index offset by the given amount.
    pub fn offset(self, amount: usize) -> Self {
        match self {
            Self::Local(index) => Self::Local(index + amount),
            Self::Dynamic(index) => Self::Dynamic(index + amount),
            Self::Preserved(index) => Self::Preserved(index + amount),
        }
    }
}

/// Used to more efficiently represent a slice of [`IrRegister`] elements.
///
/// # Note
///
/// Can only be used if all registers in the slice are
/// contiguous, e.g. `[r4, r5, r6]`.
/// This can usually be used for the results of call instructions.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct IrRegisterSlice {
    /// The index of the first register.
    start: IrRegister,
    /// The amount of registers in the contiguous slice of registers.
    len: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subslice() {
        assert_eq!(
            IrRegisterSlice::new(IrRegister::Dynamic(0), 2)
                .sub_slice(0..1)
                .unwrap(),
            IrRegisterSlice::new(IrRegister::Dynamic(0), 1),
        );
        assert_eq!(
            IrRegisterSlice::new(IrRegister::Dynamic(0), 2)
                .sub_slice(1..2)
                .unwrap(),
            IrRegisterSlice::new(IrRegister::Dynamic(1), 1),
        );
        assert_eq!(
            IrRegisterSlice::new(IrRegister::Dynamic(0), 2)
                .sub_slice(0..2)
                .unwrap(),
            IrRegisterSlice::new(IrRegister::Dynamic(0), 2),
        );
    }
}

impl IrRegisterSlice {
    /// TODO: remove again
    pub fn empty() -> Self {
        Self {
            start: IrRegister::Local(0),
            len: 0,
        }
    }

    /// Creates an [`IrRegisterSlice`] that is a sub slice of `self`.
    ///
    /// Returns `None` if the range is out of bounds.
    pub fn sub_slice(self, range: Range<usize>) -> Option<Self> {
        let start = self.first().unwrap_or(IrRegister::Dynamic(0));
        let len = self.len() as usize;
        if len < range.end {
            // The subslice is out of bounds of the original slice.
            return None;
        }
        let new_start = start.offset(range.start);
        let new_len = range.len();
        if len < new_len {
            // Subslices must have a length less-than or equal to the original.
            return None;
        }
        Some(Self {
            start: new_start,
            len: new_len as u16,
        })
    }

    /// Creates a new register slice.
    pub fn new(start: IrRegister, len: u16) -> Self {
        Self { start, len }
    }

    /// Returns the length of the register slice.
    pub fn len(&self) -> u16 {
        self.len
    }

    /// Returns the starting register of the slice if the length is 1.
    ///
    /// Returns `None` otherwise.
    pub fn single_mut(&mut self) -> Option<&mut IrRegister> {
        if self.len == 1 {
            return Some(&mut self.start);
        }
        None
    }

    /// Returns the first [`IrRegister`] of the slice if the slice is non-empty.
    pub fn first(&self) -> Option<IrRegister> {
        if self.len == 0 {
            return None;
        }
        Some(self.start)
    }

    /// Returns the [`IrRegister`] at the `index` if within bounds.
    ///
    /// Returns `None` otherwise.
    pub fn get(&self, index: u16) -> Option<IrRegister> {
        if index < self.len {
            return self.start.offset(index as usize).into();
        }
        None
    }

    /// Returns an iterator over the registers of the register slice.
    pub fn iter(&self) -> IrRegisterSliceIter {
        IrRegisterSliceIter {
            slice: *self,
            current: 0,
        }
    }
}

impl IntoIterator for IrRegisterSlice {
    type Item = IrRegister;
    type IntoIter = IrRegisterSliceIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator over the registers of an [`IrRegisterSlice`].
#[derive(Debug)]
pub struct IrRegisterSliceIter {
    slice: IrRegisterSlice,
    current: u16,
}

impl Iterator for IrRegisterSliceIter {
    type Item = IrRegister;

    fn next(&mut self) -> Option<Self::Item> {
        match self.slice.get(self.current) {
            Some(register) => {
                self.current += 1;
                Some(register)
            }
            None => None,
        }
    }
}

/// A provided input for instruction construction.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IrProvider {
    /// The input is stored in a register.
    Register(IrRegister),
    /// The input is an immediate constant value.
    Immediate(UntypedValue),
}

impl From<IrRegister> for IrProvider {
    fn from(register: IrRegister) -> Self {
        Self::Register(register)
    }
}

impl From<UntypedValue> for IrProvider {
    fn from(value: UntypedValue) -> Self {
        Self::Immediate(value)
    }
}

impl IrProvider {
    /// Returns `Some` if the [`IrProvider`] is a `Register`.
    pub fn filter_register(&self) -> Option<IrRegister> {
        match self {
            IrProvider::Register(register) => Some(*register),
            IrProvider::Immediate(_) => None,
        }
    }

    /// Returns `Some` if the [`IrProvider`] is an immediate value.
    pub fn filter_immediate(&self) -> Option<UntypedValue> {
        match self {
            IrProvider::Register(_) => None,
            IrProvider::Immediate(value) => Some(*value),
        }
    }
}

/// A reference to a provider slice.
///
/// # Note
///
/// References of this type are not deduplicated.
/// The actual elements of the [`IrProviderSlice`] are
/// stored in a  [`ProviderSliceArena`].
/// Use [`ProviderSliceArena::resolve`] to access the
/// underlying providers of the [`IrProviderSlice`].
#[derive(Debug, Copy, Clone)]
pub struct IrProviderSlice {
    /// The index to the first [`IrProvider`] of the slice.
    first: u32,
    /// The number of providers in the slice.
    len: u32,
}

impl IrProviderSlice {
    /// TODO: remove again
    pub fn empty() -> Self {
        Self { first: 0, len: 0 }
    }
}

/// An arena to efficiently allocate provider slices.
///
/// # Note
///
/// This implementation does not deduplicate equivalent register slices.
#[derive(Debug, Default)]
pub struct ProviderSliceArena {
    providers: Vec<IrProvider>,
}

impl ProviderSliceArena {
    /// Allocates a new [`IrProviderSlice`] consisting of the given registers.
    pub fn alloc<T>(&mut self, registers: T) -> IrProviderSlice
    where
        T: IntoIterator<Item = IrProvider>,
    {
        let first = self.providers.len();
        self.providers.extend(registers);
        let len = self.providers.len() - first;
        let first = first.try_into().unwrap_or_else(|error| {
            panic!("out of bounds index for register slice {first}: {error}")
        });
        let len = len
            .try_into()
            .unwrap_or_else(|error| panic!("register slice with length {len} too long: {error}"));
        IrProviderSlice { first, len }
    }

    /// Resolves an [`IrProviderSlice`] to its underlying registers or immediates.
    pub fn resolve(&self, slice: IrProviderSlice) -> &[IrProvider] {
        let first = slice.first as usize;
        let len = slice.len as usize;
        &self.providers[first..first + len]
    }
}
