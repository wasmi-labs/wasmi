use crate::{
    decode::Decode,
    for_each_op,
    primitive::*,
    CheckedOpDecoder,
    DecodeError,
    Slice,
    UncheckedOpDecoder,
};

/// Trait implemented by types that can be visited by an [`Visitor`].
pub trait Visit<'op>: crate::decode::Decoder<'op> {
    /// Visits the associated method of the `visitor` for `self`.
    fn visit<V>(&mut self, visitor: &mut V) -> Result<V::Output, Self::Error>
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
        /// Visitors can be visited via [`CheckedOpDecoder::visit`] and [`UncheckedOpDecoder::visit`].
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

        impl<'op, D> Visit<'op> for D
        where
            D: crate::decode::Decoder<'op>,
        {
            fn visit<V>(&mut self, __visitor: &mut V) -> Result<V::Output, D::Error>
            where
                V: Visitor,
            {
                match crate::OpCode::decode(self)? {
                    $(
                        crate::OpCode::$camel_name => {
                            let crate::op::$camel_name { $( $( $field_ident ),* )? } =
                                <crate::op::$camel_name as crate::decode::Decode<'op>>::decode(self)?;
                            Ok(__visitor.$snake_name($( $( $field_ident ),* )?))
                        },
                    )*
                }
            }
        }
    };
}
for_each_op!(define_visitor);

impl<'op> CheckedOpDecoder<'op> {
    /// Visits `visitor` with the next decoded [`Op`](crate::Op).
    ///
    /// # Errors
    ///
    /// If decoding of the next [`Op`](crate::Op) fails.
    pub fn visit<V>(&mut self, visitor: &mut V) -> Result<V::Output, DecodeError>
    where
        V: Visitor,
    {
        self.0.visit(visitor)
    }
}

impl UncheckedOpDecoder {
    /// Visits `visitor` with the next decoded [`Op`](crate::Op).
    ///
    /// # Safety
    ///
    /// The caller has to ensure that decoding of the next [`Op`](crate::Op) does not fail.
    pub unsafe fn visit<V>(&mut self, visitor: &mut V) -> V::Output
    where
        V: Visitor,
    {
        self.0.visit(visitor).unwrap_unchecked()
    }
}
