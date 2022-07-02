//! Datastructure to efficiently store function bodies and their instructions.

use super::ExecInstruction;
use crate::arena::Index;
use alloc::vec::Vec;
use core::slice;

/// A reference to a Wasm function body stored in the [`CodeMap`].
#[derive(Debug, Copy, Clone)]
pub struct FuncBody {
    /// The offset within the [`CodeMap`] to the first instruction.
    inst: FirstInstr,
    /// The number of instructions of the [`FuncBody`].
    len_insts: u16,
    /// The number of registers that the function requires for execution.
    ///
    /// # Note
    ///
    /// This _does_ include registers for function parameters, local variables
    /// and auxiliary registers required for execution.
    len_regs: u16,
}

impl FuncBody {
    /// Creates a new [`FuncBody`].
    pub fn new(inst: FirstInstr, len_insts: u16, len_regs: u16) -> Self {
        Self {
            inst,
            len_insts,
            len_regs,
        }
    }

    /// Returns the index to the first instruction stored in the [`CodeMap`].
    ///
    /// # Note
    ///
    /// Since instruction of the same function in the [`CodeMap`] are stored
    /// consecutively the only other information required to form the entire
    /// function body is the amount of instructions of the function which is
    /// given by [`FuncBody::len`].
    ///
    /// [`FuncBody::len`]: #method.len
    pub(super) fn inst(self) -> FirstInstr {
        self.inst
    }

    /// Returns the number of instruction of the [`FuncBody`].
    pub(super) fn len_insts(self) -> u16 {
        self.len_insts
    }

    /// Returns the number of registers the function requires for execution.
    ///
    /// # Note
    ///
    /// This does include registers for parameters and local variables.
    pub(super) fn len_regs(self) -> u16 {
        self.len_regs
    }
}

/// A handle to the first [`ExecInstruction`] of a
/// [`FuncBody`] stored within a [`CodeMap`].
#[derive(Debug, Copy, Clone)]
pub struct FirstInstr(u32);

impl Index for FirstInstr {
    fn into_usize(self) -> usize {
        self.0 as usize
    }

    fn from_usize(value: usize) -> Self {
        assert!(value <= u32::MAX as usize);
        Self(value as u32)
    }
}

/// Datastructure to efficiently store Wasm function bodies.
#[derive(Debug, Default)]
pub struct CodeMap {
    /// The instructions of all allocated function bodies.
    ///
    /// By storing all `wasmi` bytecode instructions in a single
    /// allocation we avoid an indirection when calling a function
    /// compared to a solution that stores instructions of different
    /// function bodies in different allocations.
    ///
    /// Also this improves efficiency of deallocating the [`CodeMap`]
    /// and generally improves data locality.
    insts: Vec<ExecInstruction>,
}

impl CodeMap {
    /// Returns the next [`FuncBody`] index.
    fn next_index(&self) -> FirstInstr {
        FirstInstr::from_usize(self.insts.len())
    }

    /// Allocates a new function body to the [`CodeMap`].
    ///
    /// Returns a reference to the allocated function body that can
    /// be used with [`CodeMap::resolve`] in order to resolve its
    /// instructions.
    pub fn alloc<I>(&mut self, insts: I, len_regs: u16) -> FuncBody
    where
        I: IntoIterator<Item = ExecInstruction>,
    {
        let inst = self.next_index();
        let insts = insts.into_iter();
        let len_before = self.insts.len();
        self.insts.extend(insts);
        let len_after = self.insts.len();
        let len_insts = (len_after - len_before).try_into().unwrap_or_else(|error| {
            panic!(
                "tried to allocate function with too many instructions ({}): {}",
                len_before, error
            )
        });
        FuncBody::new(inst, len_insts, len_regs)
    }

    /// Resolves the instruction of the function body.
    ///
    /// # Panics
    ///
    /// If the given `func_body` is invalid for this [`CodeMap`].
    pub fn resolve(&self, func_body: FuncBody) -> ResolvedFuncBody {
        let first_inst = func_body.inst().into_usize();
        let len_insts = func_body.len_insts() as usize;
        let insts = &self.insts[first_inst..(first_inst + len_insts)];
        ResolvedFuncBody { insts }
    }
}

/// A resolved Wasm function body that is stored in a [`CodeMap`].
///
/// Allows to immutably access the `wasmi` instructions of a Wasm
/// function stored in the [`CodeMap`].
#[derive(Debug, Copy, Clone)]
pub struct ResolvedFuncBody<'a> {
    insts: &'a [ExecInstruction],
}

impl ResolvedFuncBody<'_> {
    /// Returns an iterator over the instructions of the resolved function body.
    ///
    /// # Note
    ///
    /// This API is currently in use mainly for debugging purposes.
    pub fn iter(&self) -> ResolvedFuncBodyIter {
        self.into_iter()
    }

    /// Returns the instruction at the given index.
    ///
    /// # Panics
    ///
    /// If there is no instruction at the given index.
    #[cfg(test)]
    pub fn get(&self, index: usize) -> Option<&ExecInstruction> {
        self.insts.get(index)
    }

    /// Returns the instruction at the given index.
    ///
    /// # Note
    ///
    /// This avoids bounds checking in `--release` builds.
    /// For debugging purposes those bounds checks are enabled for `--debug`
    /// builds.
    ///
    /// # Safety
    ///
    /// The caller is repsonsible to provide valid indices.
    pub unsafe fn get_release_unchecked(&self, index: usize) -> &ExecInstruction {
        debug_assert!(
            self.insts.get(index).is_some(),
            "expect to find instruction at index {} due to validation but found none",
            index
        );
        // # Safety
        //
        // This access is safe if all possible accesses have already been
        // checked during Wasm validation. Functions and their instructions including
        // jump addresses are immutable after Wasm function compilation and validation
        // and therefore this bounds check can be safely eliminated.
        //
        // Note that eliminating this bounds check is extremely valuable since this
        // part of the `wasmi` interpreter is part of the interpreter's hot path.
        self.insts.get_unchecked(index)
    }
}

impl<'a> IntoIterator for ResolvedFuncBody<'a> {
    type Item = &'a ExecInstruction;
    type IntoIter = ResolvedFuncBodyIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ResolvedFuncBodyIter {
            iter: self.insts.iter(),
        }
    }
}

/// An iterator over the instruction of a resolved function body.
#[derive(Debug)]
pub struct ResolvedFuncBodyIter<'a> {
    iter: slice::Iter<'a, ExecInstruction>,
}

impl<'a> Iterator for ResolvedFuncBodyIter<'a> {
    type Item = &'a ExecInstruction;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
