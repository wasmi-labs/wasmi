use crate::{
    ir::{Op, Slot},
    ValType,
};

/// The number of Wasmi engine cells a value of type `ty` requires to be stored.
pub fn required_cells_of_type(ty: ValType) -> u8 {
    match ty {
        ValType::V128 => 2,
        _ => 1,
    }
}

/// Bail out early in case the current code is unreachable.
///
/// # Note
///
/// - This should be prepended to most Wasm operator translation procedures.
/// - If we are in unreachable code most Wasm translation is skipped. Only
///   certain control flow operators such as `End` are going through the
///   translation process. In particular the `End` operator may end unreachable
///   code blocks.
macro_rules! bail_unreachable {
    ($this:ident) => {{
        if !$this.reachable {
            return ::core::result::Result::Ok(());
        }
    }};
}

/// Used to swap operands of binary [`Op`] constructor.
///
/// [`Op`]: crate::ir::Op
macro_rules! swap_ops {
    ($make_instr:path) => {{ |result: $crate::ir::Slot, lhs, rhs| -> $crate::ir::Op { $make_instr(result, rhs, lhs) } }};
}

/// Implemented by types that can be reset for reuse.
pub trait Reset: Sized {
    /// Resets `self` for reuse.
    fn reset(&mut self);

    /// Returns `self` in reset state.
    #[must_use]
    fn into_reset(self) -> Self {
        let mut this = self;
        this.reset();
        this
    }
}

/// Types that have reusable heap allocations.
pub trait ReusableAllocations {
    /// The type of the reusable heap allocations.
    type Allocations: Default + Reset;

    /// Returns the reusable heap allocations of `self`.
    fn into_allocations(self) -> Self::Allocations;
}

/// A concrete input to a Wasmi instruction.
pub enum Input<T> {
    /// A [`Slot`] operand.
    Slot(Slot),
    /// A 16-bit encoded immediate value operand.
    Immediate(T),
}

/// Extension trait to update the result [`Slot`] of an [`Op`].
pub trait UpdateResultSlot: Sized {
    /// Updates the result [`Slot`] of `self` if possible.
    ///
    /// # Note
    ///
    /// - Returns `Some` resulting `Self` if the update was successful.
    /// - Returns `None` if the result update could not be applied.
    fn update_result_slot(&self, new_result: Slot) -> Option<Self>;
}

impl UpdateResultSlot for Op {
    fn update_result_slot(&self, new_result: Slot) -> Option<Self> {
        let mut op = *self;
        let result_mut = op.result_mut()?;
        *result_mut = new_result;
        Some(op)
    }
}
