use crate::*;

/// Trait implemented by types that can be visited by an [`Visitor`].
pub trait Visit: Sized {
    /// Visits the associated method of the `visitor` for `self`.
    fn visit<V>(self, visitor: &mut V) -> V::Output
    where
        V: Visitor;
}

macro_rules! define_visitor {
    (
        $(
            $( #[doc = $doc:literal] )*
            #[snake_name($snake_name:ident)]
            $camel_name:ident $(<$lt:lifetime>)? $( {
                $(
                    $( #[$field_attr:meta ] )*
                    $field_ident:ident: $field_ty:ty
                ),* $(,)?
            } )?
        ),* $(,)?
    ) => {
        /// Trait implemented by Wasmi operator visitors.
        ///
        /// Types implementing [`Visit`] can be visited by types implementing [`Visitor`].
        pub trait Visitor {
            /// The result type returned by each visit method of the visitor.
            type Output;

            $(
                $( #[doc = $doc] )*
                fn $snake_name $(<$lt>)? (&mut self, $(
                    $(
                        $field_ident: $field_ty
                    ),*
                )? ) -> Self::Output;
            )*
        }

        impl<'op> crate::Visit for crate::Op<'op> {
            fn visit<V>(self, visitor: &mut V) -> V::Output
            where
                V: crate::Visitor,
            {
                match self {
                    $(
                        Self::$camel_name(crate::op::$camel_name { $( $( $field_ident ),* )? }) => {
                            visitor.$snake_name(
                                $( $( $field_ident ),* )?
                            )
                        }
                    )*
                }
            }
        }

        $(
            impl$(<$lt>)? crate::Visit for crate::op::$camel_name $(<$lt>)? {
                fn visit<V>(self, visitor: &mut V) -> V::Output
                where
                    V: crate::Visitor,
                {
                    visitor.$snake_name(
                        $( $( self.$field_ident ),* )?
                    )
                }
            }
        )*
    };
}
for_each_op!(define_visitor);
