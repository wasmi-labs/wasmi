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
        /// This type represents all such words and for simplicity we call the type [`Instruction`], still.
        ///
        /// Most instructions are composed of a single instruction word. An example of
        /// this is [`Instruction::I32Add`]. However, some instructions, like
        /// [`Instruction::Select`], are composed of two or more instruction words.
        ///
        /// The Wasmi bytecode translation makes sure that instructions always appear in valid sequences.
        /// The Wasmi executor relies on the guarantees that the Wasmi translator provides.
        ///
        /// The documentation of each [`Instruction`] describes its encoding in the
        /// `#Encoding` section of its documentation if it requires more than a single
        /// instruction for its encoding.
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        #[non_exhaustive]
        #[repr(u16)]
        pub enum Instruction {
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

        impl Instruction {
            $(
                #[doc = concat!("Creates a new [`Instruction::", stringify!($name), "`].")]
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
    };
}
for_each_op::for_each_op!(define_enum);

/// Helper trait for [`Instruction::result`] method implementation.
trait IntoReg: Sized {
    /// Converts `self` into a [`Reg`] if possible.
    fn into_reg(self) -> Option<Reg> {
        None
    }
}

impl IntoReg for Reg {
    fn into_reg(self) -> Option<Reg> {
        Some(self)
    }
}
impl IntoReg for [Reg; 2] {}
impl IntoReg for RegSpan {}
impl<const N: u16> IntoReg for FixedRegSpan<N> {}
impl IntoReg for () {}

macro_rules! define_result {
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
        impl Instruction {
            /// Returns the result [`Reg`] for `self`.
            ///
            /// Returns `None` if `self` does not statically return a single [`Reg`].
            pub fn result(&self) -> Option<$crate::Reg> {
                match *self {
                    $(
                        Self::$name { $( $( $result_name, )? )* .. } => {
                            IntoReg::into_reg((
                                $( $( $result_name )? )*
                            ))
                        }
                    )*
                }
            }
        }
    };
}
for_each_op::for_each_op!(define_result);

impl Instruction {
    /// Creates a new [`Instruction::ReturnReg2`] for the given [`Reg`] indices.
    pub fn return_reg2_ext(reg0: impl Into<Reg>, reg1: impl Into<Reg>) -> Self {
        Self::return_reg2([reg0.into(), reg1.into()])
    }

    /// Creates a new [`Instruction::ReturnReg3`] for the given [`Reg`] indices.
    pub fn return_reg3_ext(
        reg0: impl Into<Reg>,
        reg1: impl Into<Reg>,
        reg2: impl Into<Reg>,
    ) -> Self {
        Self::return_reg3([reg0.into(), reg1.into(), reg2.into()])
    }

    /// Creates a new [`Instruction::ReturnMany`] for the given [`Reg`] indices.
    pub fn return_many_ext(
        reg0: impl Into<Reg>,
        reg1: impl Into<Reg>,
        reg2: impl Into<Reg>,
    ) -> Self {
        Self::return_many([reg0.into(), reg1.into(), reg2.into()])
    }

    /// Creates a new [`Instruction::Copy2`].
    pub fn copy2_ext(results: RegSpan, value0: impl Into<Reg>, value1: impl Into<Reg>) -> Self {
        let span = FixedRegSpan::new(results).unwrap_or_else(|_| {
            panic!("encountered invalid `results` `RegSpan` for `Copy2`: {results:?}")
        });
        Self::copy2(span, [value0.into(), value1.into()])
    }

    /// Creates a new [`Instruction::CopyMany`].
    pub fn copy_many_ext(results: RegSpan, head0: impl Into<Reg>, head1: impl Into<Reg>) -> Self {
        Self::copy_many(results, [head0.into(), head1.into()])
    }

    /// Creates a new [`Instruction::CopyManyNonOverlapping`].
    pub fn copy_many_non_overlapping_ext(
        results: RegSpan,
        head0: impl Into<Reg>,
        head1: impl Into<Reg>,
    ) -> Self {
        Self::copy_many_non_overlapping(results, [head0.into(), head1.into()])
    }

    /// Creates a new [`Instruction::Register2`] instruction parameter.
    pub fn register2_ext(reg0: impl Into<Reg>, reg1: impl Into<Reg>) -> Self {
        Self::register2([reg0.into(), reg1.into()])
    }

    /// Creates a new [`Instruction::Register3`] instruction parameter.
    pub fn register3_ext(reg0: impl Into<Reg>, reg1: impl Into<Reg>, reg2: impl Into<Reg>) -> Self {
        Self::register3([reg0.into(), reg1.into(), reg2.into()])
    }

    /// Creates a new [`Instruction::RegisterList`] instruction parameter.
    pub fn register_list_ext(
        reg0: impl Into<Reg>,
        reg1: impl Into<Reg>,
        reg2: impl Into<Reg>,
    ) -> Self {
        Self::register_list([reg0.into(), reg1.into(), reg2.into()])
    }

    /// Creates a new [`Instruction::RegisterAndImm32`] from the given `reg` and `offset_hi`.
    pub fn register_and_offset_hi(reg: impl Into<Reg>, offset_hi: Offset64Hi) -> Self {
        Self::register_and_imm32(reg, offset_hi.0)
    }

    /// Returns `Some` [`Reg`] and [`Offset64Hi`] if encoded properly.
    ///
    /// # Errors
    ///
    /// Returns back `self` if it was an incorrect [`Instruction`].
    /// This allows for a better error message to inform the user.
    pub fn filter_register_and_offset_hi(self) -> Result<(Reg, Offset64Hi), Self> {
        if let Instruction::RegisterAndImm32 { reg, imm } = self {
            return Ok((reg, Offset64Hi(u32::from(imm))));
        }
        Err(self)
    }

    /// Creates a new [`Instruction::RegisterAndImm32`] from the given `reg` and `offset_hi`.
    pub fn register_and_lane<LaneType>(reg: impl Into<Reg>, lane: LaneType) -> Self
    where
        LaneType: Into<u8>,
    {
        Self::register_and_imm32(reg, u32::from(lane.into()))
    }

    /// Returns `Some` [`Reg`] and a `lane` index if encoded properly.
    ///
    /// # Errors
    ///
    /// Returns back `self` if it was an incorrect [`Instruction`].
    /// This allows for a better error message to inform the user.
    pub fn filter_register_and_lane<LaneType>(self) -> Result<(Reg, LaneType), Self>
    where
        LaneType: TryFrom<u8>,
    {
        if let Instruction::RegisterAndImm32 { reg, imm } = self {
            let lane_index = u32::from(imm) as u8;
            let Ok(lane) = LaneType::try_from(lane_index) else {
                panic!("encountered out of bounds lane index: {}", lane_index)
            };
            return Ok((reg, lane));
        }
        Err(self)
    }

    /// Creates a new [`Instruction::Imm16AndImm32`] from the given `value` and `offset_hi`.
    pub fn imm16_and_offset_hi(value: impl Into<AnyConst16>, offset_hi: Offset64Hi) -> Self {
        Self::imm16_and_imm32(value, offset_hi.0)
    }

    /// Returns `Some` [`Reg`] and [`Offset64Hi`] if encoded properly.
    ///
    /// # Errors
    ///
    /// Returns back `self` if it was an incorrect [`Instruction`].
    /// This allows for a better error message to inform the user.
    pub fn filter_imm16_and_offset_hi<T>(self) -> Result<(T, Offset64Hi), Self>
    where
        T: From<AnyConst16>,
    {
        if let Instruction::Imm16AndImm32 { imm16, imm32 } = self {
            return Ok((T::from(imm16), Offset64Hi(u32::from(imm32))));
        }
        Err(self)
    }

    /// Creates a new [`Instruction::Imm16AndImm32`] from the given `lane` and `memory` index.
    pub fn lane_and_memory_index(value: impl Into<u8>, memory: Memory) -> Self {
        Self::imm16_and_imm32(u16::from(value.into()), u32::from(memory))
    }

    /// Returns `Some` lane and [`index::Memory`] if encoded properly.
    ///
    /// # Errors
    ///
    /// Returns back `self` if it was an incorrect [`Instruction`].
    /// This allows for a better error message to inform the user.
    pub fn filter_lane_and_memory<LaneType>(self) -> Result<(LaneType, index::Memory), Self>
    where
        LaneType: TryFrom<u8>,
    {
        if let Instruction::Imm16AndImm32 { imm16, imm32 } = self {
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
    assert_eq!(::core::mem::size_of::<Instruction>(), 8);
    assert_eq!(::core::mem::align_of::<Instruction>(), 4);
}
