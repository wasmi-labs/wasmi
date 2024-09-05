use crate::{core::TrapCode, for_each_op, index::*, *};
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
        /// Actually Wasmi instructions are composed of so-called instruction words.
        /// In fact this type represents single instruction words but for simplicity
        /// we call the type [`Instruction`] still.
        /// Most instructions are composed of a single instruction words. An example of
        /// this is [`Instruction::I32Add`]. However, some instructions like
        /// [`Instruction::Select`] are composed of two or more instruction words.
        /// The Wasmi bytecode translation phase makes sure that those instruction words
        /// always appear in valid sequences. The Wasmi executor relies on this guarantee.
        /// The documentation of each [`Instruction`] variant describes its encoding in the
        /// `#Encoding` section of its documentation if it requires more than a single
        /// instruction word for its encoding.
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
