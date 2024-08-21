use crate::*;

/// Trait to query the [`OpCode`] of a Wasmi operator.
pub trait Code {
    /// Returns the [`OpCode`] of `self`.
    fn code(&self) -> OpCode;
}

macro_rules! define_enum {
    (
        $(
            $( #[doc = $doc:literal] )*
            #[snake_name($snake_name:ident)]
            $camel_name:ident $(< $lt:lifetime >)? $( {
                $(
                    $( #[$field_attr:meta ] )*
                    $field_ident:ident: $field_ty:ty
                ),* $(,)?
            } )?
        ),* $(,)?
    ) => {
        /// The `enum` operator of a Wasmi instruction.
        #[derive(
            ::core::fmt::Debug,
            ::core::marker::Copy,
            ::core::clone::Clone,
            ::core::cmp::PartialEq,
            ::core::cmp::Eq,
        )]
        pub enum Op<'op> {
            $(
                $( #[doc = $doc] )*
                $camel_name(crate::op::$camel_name $(<$lt>)?),
            )*
        }

        impl crate::Code for Op<'_> {
            fn code(&self) -> crate::OpCode {
                match self {
                    $(
                        Self::$camel_name { .. } => OpCode::$camel_name
                    ),*
                }
            }
        }

        /// The `enum` operator of a Wasmi instruction.
        #[derive(
            ::core::fmt::Debug,
            ::core::cmp::PartialEq,
            ::core::cmp::Eq,
        )]
        pub enum OpMut<'op> {
            $(
                $( #[doc = $doc] )*
                $camel_name(crate::op::r#mut::$camel_name<'op>),
            )*
        }

        impl crate::Code for OpMut<'_> {
            fn code(&self) -> crate::OpCode {
                match self {
                    $(
                        Self::$camel_name { .. } => OpCode::$camel_name
                    ),*
                }
            }
        }

        /// The op-code of a Wasmi instruction.
        #[derive(
            ::core::fmt::Debug,
            ::core::marker::Copy,
            ::core::clone::Clone,
            ::core::cmp::PartialEq,
            ::core::cmp::Eq,
            ::core::cmp::PartialOrd,
            ::core::cmp::Ord,
        )]
        #[repr(u16)]
        pub enum OpCode {
            $(
                $( #[doc = $doc] )*
                $camel_name
            ),*
        }

        /// Wasmi bytecode operator definitions.
        pub mod op {
            use crate::*;

            $(
                $( #[doc = $doc] )*
                #[derive(
                    ::core::fmt::Debug,
                    ::core::marker::Copy,
                    ::core::clone::Clone,
                    ::core::cmp::PartialEq,
                    ::core::cmp::Eq,
                )]
                pub struct $camel_name $(<$lt>)? { $(
                    $(
                        $( #[$field_attr] )*
                        pub $field_ident: $field_ty
                    ),*
                )? }

                impl$(<$lt>)? Code for $camel_name $(<$lt>)? {
                    fn code(&self) -> crate::OpCode {
                        crate::OpCode::$camel_name
                    }
                }

                impl<'op> From<$camel_name $(<$lt>)?> for crate::Op<'op> {
                    fn from(__value: $camel_name $(<$lt>)?) -> Self {
                        Self::$camel_name(__value)
                    }
                }
            )*

            pub mod r#mut {
                use crate::*;

                $(
                    $( #[doc = $doc] )*
                    #[derive(
                        ::core::fmt::Debug,
                        ::core::cmp::PartialEq,
                        ::core::cmp::Eq,
                    )]
                    pub struct $camel_name<'op> {
                        $(
                            $(
                                $( #[$field_attr] )*
                                pub $field_ident: <$field_ty as crate::decode::Mut<'op>>::Type,
                            )*
                        )?
                        pub(crate) __marker: ::core::marker::PhantomData<&'op mut ()>,
                    }

                    impl<'op> Code for $camel_name<'op> {
                        fn code(&self) -> crate::OpCode {
                            crate::OpCode::$camel_name
                        }
                    }

                    impl<'op> From<$camel_name<'op>> for crate::OpMut<'op> {
                        fn from(__value: $camel_name<'op>) -> Self {
                            Self::$camel_name(__value)
                        }
                    }
                )*
            }
        }
    };
}
for_each_op!(define_enum);
