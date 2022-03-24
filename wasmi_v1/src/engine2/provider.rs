use super::{bytecode::Register, ConstRef};
use crate::arena::Index;
use alloc::collections::{btree_map::Entry, BTreeMap};
use core::ops::Neg;

#[derive(Debug)]
pub struct DedupProviderSliceArena {
    scratch: Vec<Provider>,
    dedup: BTreeMap<Box<[Provider]>, DedupProviderSlice>,
    providers: Vec<Provider>,
}

impl Default for DedupProviderSliceArena {
    fn default() -> Self {
        Self {
            scratch: Vec::default(),
            dedup: BTreeMap::new(),
            providers: Vec::default(),
        }
    }
}

impl DedupProviderSliceArena {
    // /// Allocates a new [`RegisterSlice`] consisting of the given registers.
    pub fn alloc<T>(&mut self, registers: T) -> DedupProviderSlice
    where
        T: IntoIterator<Item = Provider>,
    {
        self.scratch.clear();
        self.scratch.extend(registers);
        match self.dedup.entry(self.scratch.clone().into_boxed_slice()) {
            Entry::Occupied(occupied) => *occupied.get(),
            Entry::Vacant(vacant) => {
                let first = self.providers.len();
                self.providers.extend_from_slice(&self.scratch[..]);
                let len = self.providers.len() - first;
                let first = first.try_into().unwrap_or_else(|error| {
                    panic!("out of bounds index for register slice: {}", first)
                });
                let len = len
                    .try_into()
                    .unwrap_or_else(|error| panic!("register slice too long: {}", len));
                let dedup = DedupProviderSlice { first, len };
                vacant.insert(dedup);
                dedup
            }
        }
    }

    /// Resolves a [`RegisterSlice`] to its underlying registers or immediates.
    pub fn resolve(&self, slice: DedupProviderSlice) -> &[Provider] {
        let first = slice.first as usize;
        let len = slice.len as usize;
        &self.providers[first..first + len]
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DedupProviderSlice {
    first: u16,
    len: u16,
}

impl DedupProviderSlice {
    pub fn new(first: u16, len: u16) -> Self {
        Self { first, len }
    }

    pub fn empty() -> Self {
        Self { first: 0, len: 0 }
    }
}

/// Either a [`Register`] or an [`Immediate`] input value.
///
/// # Developer Note
///
/// Negative numbers represent an index into the constant table
/// and positive numbers represent the index of a register.
/// Both, indices into the constant table and indices of registers
/// are `u16`, therefore it is possible to represent them using a
/// value of type `i32`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Provider(i32);

impl From<Register> for Provider {
    fn from(register: Register) -> Self {
        Self::from_register(register)
    }
}

impl From<ConstRef> for Provider {
    fn from(const_ref: ConstRef) -> Self {
        Self::from_immediate(const_ref)
    }
}

impl Provider {
    pub fn from_register(register: Register) -> Self {
        let inner = register.into_inner() as u32 as i32;
        Self(inner)
    }

    pub fn from_immediate(immediate: ConstRef) -> Self {
        let inner = (u32::from(immediate.into_inner()) as i32).neg();
        Self(inner)
    }
}

impl Provider {
    pub fn decode(self) -> RegisterOrImmediate {
        if self.0.is_negative() {
            return ConstRef::from_usize(self.0.abs() as usize).into();
        }
        Register::from_inner(self.0 as u16).into()
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RegisterOrImmediate {
    Register(Register),
    Immediate(ConstRef),
}

impl From<Register> for RegisterOrImmediate {
    fn from(register: Register) -> Self {
        Self::Register(register)
    }
}

impl From<ConstRef> for RegisterOrImmediate {
    fn from(immediate: ConstRef) -> Self {
        Self::Immediate(immediate)
    }
}
