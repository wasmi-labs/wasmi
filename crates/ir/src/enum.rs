#[cfg(feature = "simd")]
use crate::core::simd::{ImmLaneIdx16, ImmLaneIdx2, ImmLaneIdx4, ImmLaneIdx8};
#[cfg(all(feature = "simd", doc))]
use crate::core::V128;
use crate::{core::TrapCode, index::*, primitive::Offset64Hi, *};
use ::core::num::{NonZeroI32, NonZeroI64, NonZeroU32, NonZeroU64};

macro_rules! define_enum {
    (
        $(
            $( #[doc = $doc:literal] )*
            #[snake_name($snake_name:ident)]
            $name:ident
            $(
                {
                    $(
                        @ $result_name:ident: $result_ty:ty,
                    )?
                    $(
                        $( #[$field_docs:meta] )*
                        $field_name:ident: $field_ty:ty
                    ),*
                    $(,)?
                }
            )?
        ),* $(,)?
    ) => {
        /// A Wasmi instruction.
        ///
        /// Wasmi instructions are composed of so-called instruction words.
        /// This type represents all such words and for simplicity we call the type [`Op`], still.
        ///
        /// Most instructions are composed of a single instruction word. An example of
        /// this is [`Op::I32Add`]. However, some instructions, like the `select` instructions
        /// are composed of two or more instruction words.
        ///
        /// The Wasmi bytecode translation makes sure that instructions always appear in valid sequences.
        /// The Wasmi executor relies on the guarantees that the Wasmi translator provides.
        ///
        /// The documentation of each [`Op`] describes its encoding in the
        /// `#Encoding` section of its documentation if it requires more than a single
        /// instruction for its encoding.
        #[derive(Debug)]
        #[non_exhaustive]
        #[repr(u16)]
        pub enum Op {
            $(
                $( #[doc = $doc] )*
                $name
                $(
                    {
                        $(
                            /// The register(s) storing the result of the instruction.
                            $result_name: $result_ty,
                        )?
                        $(
                            $( #[$field_docs] )*
                            $field_name: $field_ty
                        ),*
                    }
                )?
            ),*
        }

        impl Op {
            $(
                #[doc = concat!("Creates a new [`Op::", stringify!($name), "`].")]
                pub fn $snake_name(
                    $(
                        $( $result_name: impl Into<$result_ty>, )?
                        $( $field_name: impl Into<$field_ty> ),*
                    )?
                ) -> Self {
                    Self::$name {
                        $(
                            $( $result_name: $result_name.into(), )?
                            $( $field_name: $field_name.into() ),*
                        )?
                    }
                }
            )*
        }

        impl<'a> $crate::visit_results::ResultsVisitor for &'a mut Op {
            fn host_visitor<V: VisitResults>(self, visitor: &mut V) {
                match self {
                    $(
                        Op::$name { $( $( $result_name, )? .. )? } => {
                            $(
                                $( $result_name.host_visitor(visitor); )?
                            )?
                        }
                    )*
                }
            }
        }
    };
}
for_each_op::for_each_op!(define_enum);

impl Copy for Op {}
impl Clone for Op {
    fn clone(&self) -> Self {
        *self
    }
}

impl Op {
    /// Creates a new [`Op::ReturnSlot2`] for the given [`Slot`] indices.
    pub fn return_reg2_ext(reg0: impl Into<Slot>, reg1: impl Into<Slot>) -> Self {
        Self::return_reg2([reg0.into(), reg1.into()])
    }

    /// Creates a new [`Op::ReturnSlot3`] for the given [`Slot`] indices.
    pub fn return_reg3_ext(
        reg0: impl Into<Slot>,
        reg1: impl Into<Slot>,
        reg2: impl Into<Slot>,
    ) -> Self {
        Self::return_reg3([reg0.into(), reg1.into(), reg2.into()])
    }

    /// Creates a new [`Op::ReturnMany`] for the given [`Slot`] indices.
    pub fn return_many_ext(
        reg0: impl Into<Slot>,
        reg1: impl Into<Slot>,
        reg2: impl Into<Slot>,
    ) -> Self {
        Self::return_many([reg0.into(), reg1.into(), reg2.into()])
    }

    /// Creates a new [`Op::Copy2`].
    pub fn copy2_ext(results: SlotSpan, value0: impl Into<Slot>, value1: impl Into<Slot>) -> Self {
        let span = FixedSlotSpan::new(results).unwrap_or_else(|_| {
            panic!("encountered invalid `results` `SlotSpan` for `Copy2`: {results:?}")
        });
        Self::copy2(span, [value0.into(), value1.into()])
    }

    /// Creates a new [`Op::CopyMany`].
    pub fn copy_many_ext(
        results: SlotSpan,
        head0: impl Into<Slot>,
        head1: impl Into<Slot>,
    ) -> Self {
        Self::copy_many(results, [head0.into(), head1.into()])
    }

    /// Creates a new [`Op::Slot2`] instruction parameter.
    pub fn slot2_ext(reg0: impl Into<Slot>, reg1: impl Into<Slot>) -> Self {
        Self::slot2([reg0.into(), reg1.into()])
    }

    /// Creates a new [`Op::Slot3`] instruction parameter.
    pub fn slot3_ext(reg0: impl Into<Slot>, reg1: impl Into<Slot>, reg2: impl Into<Slot>) -> Self {
        Self::slot3([reg0.into(), reg1.into(), reg2.into()])
    }

    /// Creates a new [`Op::SlotList`] instruction parameter.
    pub fn slot_list_ext(
        reg0: impl Into<Slot>,
        reg1: impl Into<Slot>,
        reg2: impl Into<Slot>,
    ) -> Self {
        Self::slot_list([reg0.into(), reg1.into(), reg2.into()])
    }

    /// Creates a new [`Op::SlotAndImm32`] from the given `reg` and `offset_hi`.
    pub fn slot_and_offset_hi(reg: impl Into<Slot>, offset_hi: Offset64Hi) -> Self {
        Self::slot_and_imm32(reg, offset_hi.0)
    }

    /// Returns `Some` [`Slot`] and [`Offset64Hi`] if encoded properly.
    ///
    /// # Errors
    ///
    /// Returns back `self` if it was an incorrect [`Op`].
    /// This allows for a better error message to inform the user.
    pub fn filter_register_and_offset_hi(self) -> Result<(Slot, Offset64Hi), Self> {
        if let Op::SlotAndImm32 { reg, imm } = self {
            return Ok((reg, Offset64Hi(u32::from(imm))));
        }
        Err(self)
    }

    /// Creates a new [`Op::SlotAndImm32`] from the given `reg` and `offset_hi`.
    pub fn slot_and_lane<LaneType>(reg: impl Into<Slot>, lane: LaneType) -> Self
    where
        LaneType: Into<u8>,
    {
        Self::slot_and_imm32(reg, u32::from(lane.into()))
    }

    /// Returns `Some` [`Slot`] and a `lane` index if encoded properly.
    ///
    /// # Errors
    ///
    /// Returns back `self` if it was an incorrect [`Op`].
    /// This allows for a better error message to inform the user.
    pub fn filter_register_and_lane<LaneType>(self) -> Result<(Slot, LaneType), Self>
    where
        LaneType: TryFrom<u8>,
    {
        if let Op::SlotAndImm32 { reg, imm } = self {
            let lane_index = u32::from(imm) as u8;
            let Ok(lane) = LaneType::try_from(lane_index) else {
                panic!("encountered out of bounds lane index: {}", lane_index)
            };
            return Ok((reg, lane));
        }
        Err(self)
    }

    /// Creates a new [`Op::Imm16AndImm32`] from the given `value` and `offset_hi`.
    pub fn imm16_and_offset_hi(value: impl Into<AnyConst16>, offset_hi: Offset64Hi) -> Self {
        Self::imm16_and_imm32(value, offset_hi.0)
    }

    /// Returns `Some` [`Slot`] and [`Offset64Hi`] if encoded properly.
    ///
    /// # Errors
    ///
    /// Returns back `self` if it was an incorrect [`Op`].
    /// This allows for a better error message to inform the user.
    pub fn filter_imm16_and_offset_hi<T>(self) -> Result<(T, Offset64Hi), Self>
    where
        T: From<AnyConst16>,
    {
        if let Op::Imm16AndImm32 { imm16, imm32 } = self {
            return Ok((T::from(imm16), Offset64Hi(u32::from(imm32))));
        }
        Err(self)
    }

    /// Creates a new [`Op::Imm16AndImm32`] from the given `lane` and `memory` index.
    pub fn lane_and_memory_index(value: impl Into<u8>, memory: Memory) -> Self {
        Self::imm16_and_imm32(u16::from(value.into()), u32::from(memory))
    }

    /// Returns `Some` lane and [`index::Memory`] if encoded properly.
    ///
    /// # Errors
    ///
    /// Returns back `self` if it was an incorrect [`Op`].
    /// This allows for a better error message to inform the user.
    pub fn filter_lane_and_memory<LaneType>(self) -> Result<(LaneType, index::Memory), Self>
    where
        LaneType: TryFrom<u8>,
    {
        if let Op::Imm16AndImm32 { imm16, imm32 } = self {
            let Ok(lane) = LaneType::try_from(i16::from(imm16) as u16 as u8) else {
                return Err(self);
            };
            return Ok((lane, index::Memory::from(u32::from(imm32))));
        }
        Err(self)
    }
}

#[test]
fn size_of() {
    // Note: In case this test starts failing:
    //
    // There currently is a bug in the Rust compiler that causes
    // Rust `enum` definitions with `#[repr(uN)]` to be incorrectly
    // sized: https://github.com/rust-lang/rust/issues/53657
    //
    // Until that bug is fixed we need to order the `enum` variant
    // fields in a precise order to end up with the correct `enum` size.
    assert_eq!(::core::mem::size_of::<Op>(), 8);
    assert_eq!(::core::mem::align_of::<Op>(), 4);
}
