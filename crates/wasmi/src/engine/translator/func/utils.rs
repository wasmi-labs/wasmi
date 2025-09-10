use crate::ir::{Const16, Const32, Slot};

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
    ($make_instr:path) => {{
        |result: $crate::ir::Slot, lhs, rhs| -> $crate::ir::Op { $make_instr(result, rhs, lhs) }
    }};
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

/// A 16-bit encoded input to Wasmi instruction.
pub type Input16<T> = Input<Const16<T>>;

/// A 32-bit encoded input to Wasmi instruction.
pub type Input32<T> = Input<Const32<T>>;

/// A concrete input to a Wasmi instruction.
pub enum Input<T> {
    /// A [`Slot`] operand.
    Slot(Slot),
    /// A 16-bit encoded immediate value operand.
    Immediate(T),
}

pub trait IntoShiftAmount {
    /// The type denoting the shift amount.
    ///
    /// This is an unsigned integer ranging from `1..N` where `N` is the number of bits in `Self`.
    type Value: Copy;

    /// Returns `self` wrapped into a proper shift amount for `Self`.
    ///
    /// Returns `None` if the resulting shift amount is 0, a.k.a. a no-op.
    fn into_shift_amount(self) -> Option<Self::Value>;
}

macro_rules! impl_into_shift_amount {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl IntoShiftAmount for $ty {
                type Value = u8;

                fn into_shift_amount(self) -> Option<Self::Value> {
                    let len_bits = (::core::mem::size_of::<Self>() * 8) as Self;
                    self.checked_rem_euclid(len_bits)
                }
            }
        )*
    };
}
impl_into_shift_amount!(i32, u32, i64, u64, i128, u128);
