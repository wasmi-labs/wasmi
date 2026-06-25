use crate::ir::{Op, SlotSpan};

/// Represents fused copy [`Op`] for copy fusion in the translator.
#[derive(Copy, Clone)]
pub struct FusedCopy {
    /// The result slots of the fused copy [`Op`].
    results: SlotSpan,
    /// The value slots of the fused copy [`Op`].
    values: SlotSpan,
    /// The number of copied slots of the fused copy [`Op`].
    len: u16,
}

impl FusedCopy {
    /// Creates a new [`FusedCopy`] from its raw parts.
    fn new(results: SlotSpan, values: SlotSpan, len: u16) -> Self {
        Self {
            results,
            values,
            len,
        }
    }

    /// Returns `Some` if `op` is a copy [`Op`] that can be fused.
    ///
    /// Otherwise returns `None`.
    pub fn from_op(op: Op) -> Option<Self> {
        let (results, values, len) = match op {
            Op::U64Copy_Ss { result, value } => (SlotSpan::new(result), SlotSpan::new(value), 1),
            #[cfg(feature = "simd")]
            Op::V128Copy_Ss { result, value } => (SlotSpan::new(result), SlotSpan::new(value), 2),
            Op::CopySpanAsc {
                results,
                values,
                len,
            }
            | Op::CopySpanDes {
                results,
                values,
                len,
            } => (results, values, len),
            _ => return None,
        };
        Some(Self::new(results, values, len))
    }

    /// Returns `Some` if `self` and the `new` copy-span [`Op`] can be fused.
    ///
    /// Otherwise returns `None`.
    pub fn try_fuse(self, new: Self) -> Option<Self> {
        if let Some(fused) = self.try_fuse_copy_asc(new) {
            return Some(fused);
        }
        if let Some(fused) = self.try_fuse_copy_des(new) {
            return Some(fused);
        }
        None
    }

    /// Returns `Some` if `self` and the `new` copy-span [`Op`] can be fused in ascending slot-order.
    ///
    /// Otherwise returns `None`.
    fn try_fuse_copy_asc(self, new: Self) -> Option<Self> {
        let can_fuse = new.results.head() == self.results.head().next_n(self.len)
            && new.values.head() == self.values.head().next_n(self.len);
        if can_fuse {
            // Case: copy fusion in ascending slot order can be applied.
            return Some(Self::new(self.results, self.values, self.len + new.len));
        }
        None
    }

    /// Returns `Some` if `self` and the `new` copy-span [`Op`] can be fused in descending slot-order.
    ///
    /// Otherwise returns `None`.
    fn try_fuse_copy_des(self, new: Self) -> Option<Self> {
        let can_fuse = new.results.head() == self.results.head().prev_n(new.len)
            && new.values.head() == self.values.head().prev_n(new.len);
        if can_fuse {
            // Case: copy fusion in descending slot order can be applied.
            return Some(Self::new(new.results, new.values, self.len + new.len));
        }
        None
    }

    /// Lowers `self` back into an [`Op`] for encoding.
    ///
    /// This returns the most efficient [`Op`] that preserves copy semantics.
    pub fn into_op(self) -> Op {
        let Self {
            results,
            values,
            len,
        } = self;
        match len {
            1 => Op::u64_copy_ss(results.head(), values.head()),
            #[cfg(feature = "simd")]
            2 => Op::v128_copy_ss(results.head(), values.head()),
            _ if results < values => Op::copy_span_asc(results, values, len),
            _ => Op::copy_span_des(results, values, len),
        }
    }
}
