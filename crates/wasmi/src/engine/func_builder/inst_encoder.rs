//! Abstractions to build up instructions forming Wasm function bodies.

use core::{mem, ptr};
use super::{
    labels::{LabelRef, LabelRegistry},
    Instr,
    RelativeDepth,
    super::OpCode,
};
use crate::engine::{
    bytecode::{BranchOffset, Instruction},
};
use alloc::vec::Vec;

/// The buffer that stores all the encoded instructions.
#[derive(Debug, Default)]
pub struct EncodedInstrs {
    encoded: Vec<u8>,
}

impl EncodedInstrs {
    /// Resets the [`EncodedInstrs`] to allow for reuse.
    pub fn reset(&mut self) {
        self.encoded.clear();
    }

    /// Returns the index of the next encoded instruction.
    pub fn next(&self) -> Instr {
        Instr::from_usize(self.encoded.len())
    }

    /// Decodes the [`Instr`] and returns an exclusive reference to it.
    fn decode_mut(&mut self, instr: Instr) -> &mut Instruction {
        let (instr, _) = Instruction::decode_mut(&mut self.encoded[instr.into_usize()..])
            .expect("must have valid encoded instruction at this position");
        instr
    }

    /// Encodes `value` and pushes it onto the buffer of encoded instructions.
    /// 
    /// # Note
    /// 
    /// This method may be called multiple times upon encoding a single instruction.
    /// For example one time for the discriminant, and another time for a parameter
    /// of the encoded instruction.
    /// It is possible to retrieve the [`Instr`] reference by calling
    /// [`EncodedInstrs::next`] before encoding the instruction.
    fn push_encoded<T: Copy>(&mut self, value: T) {
        let size = mem::size_of::<T>();
        let start = self.encoded.len();
        self.encoded.resize(start + size, 0);
        let buffer = &mut self.encoded[start..];
        let dst = buffer.as_mut_ptr() as *mut T;
        // Safety: The `dst` pointer is valid for writes since we
        //         just performed a `extend` right before this call
        //         and the `value` is of type `T: Copy` so it won't
        //         require special drop behavior later.
        //         The values overwritten by this `write_unaligned`
        //         are simple bytes and thus no `drop` needs to happen.
        unsafe { ptr::write_unaligned(dst, value) };
    }

    /// Encodes a `wasmi` [`OpCode`].
    /// 
    /// # Note
    /// 
    /// This is much more efficient than using [`InstructionEncoder::push_encoded`]
    /// and a quite common case for most of the `wasmi` instructions.
    fn push_opcode(&mut self, opcode: OpCode) {
        self.encoded.push(opcode as u8);
    }
}

impl Instruction {
    /// Decodes the `buffer` as [`Instruction`].
    ///
    /// Returns an exclusive reference to the [`Instruction`] and the number
    /// of bytes that make up the decoded [`Instruction`]. The number is returned
    /// so that the caller knows where the next encoded [`Instruction`] in the
    /// buffer starts.
    ///
    /// Returns `None` if the `buffer` cannot be decoded.
    pub fn decode_mut(encoded: &mut [u8]) -> Option<(&mut Instruction, usize)> {
        todo!()
    }
}

/// An instruction builder.
///
/// Allows to incrementally and efficiently build up the instructions
/// of a Wasm function body.
/// Can be reused to build multiple functions consecutively.
#[derive(Debug, Default)]
pub struct InstructionEncoder {
    /// The encoded instructions of the partially construction function.
    instrs: EncodedInstrs,
    /// All labels and their uses.
    labels: LabelRegistry,
}

impl InstructionEncoder {
    /// Resets the [`InstructionEncoder`] to allow for reuse.
    pub fn reset(&mut self) {
        self.instrs.reset();
        self.labels.reset();
    }

    /// Returns the index of the next encoded instruction.
    pub fn next_pc(&self) -> Instr {
        self.instrs.next()
    }

    /// Creates a new unresolved label and returns an index to it.
    pub fn new_label(&mut self) -> LabelRef {
        self.labels.new_label()
    }

    /// Resolve the label at the current instruction position.
    ///
    /// Does nothing if the label has already been resolved.
    ///
    /// # Note
    ///
    /// This is used at a position of the Wasm bytecode where it is clear that
    /// the given label can be resolved properly.
    /// This usually takes place when encountering the Wasm `End` operand for example.
    pub fn pin_label_if_unpinned(&mut self, label: LabelRef) {
        self.labels.try_pin_label(label, self.next_pc())
    }

    /// Resolve the label at the current instruction position.
    ///
    /// # Note
    ///
    /// This is used at a position of the Wasm bytecode where it is clear that
    /// the given label can be resolved properly.
    /// This usually takes place when encountering the Wasm `End` operand for example.
    ///
    /// # Panics
    ///
    /// If the label has already been resolved.
    pub fn pin_label(&mut self, label: LabelRef) {
        self.labels
            .pin_label(label, self.next_pc())
            .unwrap_or_else(|err| panic!("failed to pin label: {err}"));
    }

    /// Try resolving the `label` for the currently constructed instruction.
    ///
    /// Returns an uninitialized [`BranchOffset`] if the `label` cannot yet
    /// be resolved and defers resolution to later.
    pub fn try_resolve_label(&mut self, label: LabelRef) -> BranchOffset {
        let user = self.next_pc();
        self.try_resolve_label_for(label, user)
    }

    /// Try resolving the `label` for the given `instr`.
    ///
    /// Returns an uninitialized [`BranchOffset`] if the `label` cannot yet
    /// be resolved and defers resolution to later.
    pub fn try_resolve_label_for(&mut self, label: LabelRef, instr: Instr) -> BranchOffset {
        self.labels.try_resolve_label(label, instr)
    }

    /// Updates the branch offsets of all branch instructions inplace.
    ///
    /// # Panics
    ///
    /// If this is used before all branching labels have been pinned.
    fn update_branch_offsets(&mut self) {
        for (user, offset) in self.labels.resolved_users() {
            self.instrs.decode_mut(user).update_branch_offset(offset);
        }
    }

    /// Adds the given `delta` amount of fuel to the [`ConsumeFuel`] instruction `instr`.
    ///
    /// # Panics
    ///
    /// - If `instr` does not resolve to a [`ConsumeFuel`] instruction.
    /// - If the amount of consumed fuel for `instr` overflows.
    ///
    /// [`ConsumeFuel`]: enum.Instruction.html#variant.ConsumeFuel
    pub fn bump_fuel_consumption(&mut self, instr: Instr, delta: u64) {
        self.instrs.decode_mut(instr).bump_fuel_consumption(delta)
    }
}
