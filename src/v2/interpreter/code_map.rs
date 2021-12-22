//! Datastructure to efficiently store function bodies and their instructions.

use super::super::Index;
use super::Instruction;
use core::iter;
use alloc::vec::Vec;

/// A reference to a Wasm function body stored in the [`CodeMap`].
#[derive(Debug, Copy, Clone)]
pub struct FuncBody(usize);

impl Index for FuncBody {
    fn into_usize(self) -> usize {
        self.0
    }

    fn from_usize(value: usize) -> Self {
        FuncBody(value)
    }
}

/// Datastructure to efficiently store Wasm function bodies.
#[derive(Debug)]
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
    insts: Vec<Instruction>,
}

impl CodeMap {
    /// Returns the next [`FuncBody`] index.
    fn next_index(&self) -> FuncBody {
        FuncBody(self.insts.len())
    }

    /// Allocates a new function body to the [`CodeMap`].
    ///
    /// Returns a reference to the allocated function body that can
    /// be used with [`CodeMap::resolve`] in order to resolve its
    /// instructions.
    pub fn alloc<I>(&mut self, insts: I) -> FuncBody
    where
        I: IntoIterator<Item = Instruction>,
        I::IntoIter: ExactSizeIterator,
    {
        let idx = self.next_index();
        // We are inserting an artificial `unreachable` Wasm instruction
        // in between instructions of different function bodies as a small
        // safety precaution.
        let len = self.insts.len();
        let start = iter::once(Instruction::FuncBodyStart(len));
        let end = iter::once(Instruction::FuncBodyEnd);
        self.insts.extend(start.chain(insts).chain(end));
        idx
    }

    /// Resolves the instruction of the function body.
    ///
    /// # Panics
    ///
    /// If the given `func_body` is invalid for this [`CodeMap`].
    pub fn resolve(&self, func_body: FuncBody) -> ResolvedFuncBody {
        let offset = func_body.into_usize();
        let len = match &self.insts[offset] {
            Instruction::FuncBodyStart(len) => len,
            unexpected => panic!(
                "expected function start instruction but found: {:?}",
                unexpected
            ),
        };
        // The index of the first instruction in the function body.
        let first_inst = offset + 1;
        {
            // Assert that the end of the function instructions is
            // properly guarded with the `FuncBodyEnd` sentinel.
            //
            // This check is not needed to validate the integrity of
            // the resolution procedure and therefore the below assertion
            // is only performed in debug mode.
            let end = &self.insts[first_inst + len];
            debug_assert!(
                matches!(end, Instruction::FuncBodyEnd),
                "expected function end instruction but found: {:?}",
                end,
            );
        }
        let insts = &self.insts[first_inst..(first_inst + len)];
        ResolvedFuncBody { insts }
    }
}

/// A resolved Wasm function body that is stored in a [`CodeMap`].
///
/// Allows to immutably access the `wasmi` instructions of a Wasm
/// function stored in the [`CodeMap`].
///
/// # Dev. Note
///
/// This does not include the [`Instruction::FuncBodyStart`] and
/// [`Instruction::FuncBodyEnd`] instructions surrounding the instructions
/// of a function body in the [`CodeMap`].
#[derive(Debug, Copy, Clone)]
pub struct ResolvedFuncBody<'a> {
    insts: &'a [Instruction],
}

impl ResolvedFuncBody<'_> {
    /// Returns the instruction at the given index.
    ///
    /// # Panics
    ///
    /// If there is no instruction at the given index.
    pub fn get(&self, index: usize) -> &Instruction {
        &self.insts[index]
    }
}
