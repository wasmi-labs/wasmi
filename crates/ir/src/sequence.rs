use crate::{core::TrapCode, index::*, *};
use ::core::{
    num::{NonZeroI32, NonZeroI64, NonZeroU32, NonZeroU64},
    slice,
};
use std::{boxed::Box, vec::Vec};

/// A sequence of [`Instruction`]s.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct InstrSequence {
    /// The [`Instruction`] that make up all built instructions in sequence.
    instrs: Vec<Instruction>,
}

impl From<InstrSequence> for Vec<Instruction> {
    fn from(sequence: InstrSequence) -> Self {
        sequence.into_vec()
    }
}

impl From<InstrSequence> for Box<[Instruction]> {
    fn from(sequence: InstrSequence) -> Self {
        sequence.into_boxed_slice()
    }
}

impl InstrSequence {
    /// Returns `self` as vector of [`Instruction`]s.
    pub fn into_vec(self) -> Vec<Instruction> {
        self.instrs
    }

    /// Returns `self` as boxed slice of [`Instruction`]s.
    pub fn into_boxed_slice(self) -> Box<[Instruction]> {
        self.instrs.into_boxed_slice()
    }

    /// Returns the underlying [`Instruction`]s as shared slice.
    pub fn as_slice(&self) -> &[Instruction] {
        &self.instrs[..]
    }

    /// Returns the underlying [`Instruction`]s as mutable slice.
    pub fn as_slice_mut(&mut self) -> &mut [Instruction] {
        &mut self.instrs[..]
    }

    /// Returns the number of [`Instruction`] in `self`.
    #[inline]
    pub fn len(&self) -> usize {
        self.instrs.len()
    }

    /// Returns `true` if `self` is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the [`Instruction`] that is associated to `instr`.
    #[inline]
    pub fn get(&self, instr: Instr) -> Option<Instruction> {
        self.instrs.get(instr.0).copied()
    }

    /// Returns a mutable reference to the [`Instruction`] that is associated to `instr`.
    #[inline]
    pub fn get_mut(&mut self, instr: Instr) -> Option<&mut Instruction> {
        self.instrs.get_mut(instr.0)
    }

    /// Returns an iterator yielding the [`Instruction`] of the [`InstrSequence`].
    pub fn iter(&self) -> InstrIter {
        InstrIter::new(self)
    }

    /// Returns an iterator yielding mutable [`Instruction`] of the [`InstrSequence`].
    pub fn iter_mut(&mut self) -> InstrIterMut {
        InstrIterMut::new(self)
    }

    /// Pops the last [`Instruction`] in `self` if any.
    ///
    /// Returns `None` if `self` is empty.
    pub fn pop(&mut self) -> Option<Instruction> {
        self.instrs.pop()
    }
}

impl<'a> IntoIterator for &'a InstrSequence {
    type Item = &'a Instruction;
    type IntoIter = InstrIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut InstrSequence {
    type Item = &'a mut Instruction;
    type IntoIter = InstrIterMut<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

macro_rules! define_builder {
    (
        $(
            $( #[doc = $doc:literal] )*
            #[snake_name($snake_name:ident)]
            $name:ident
            $(
                {
                    // $( @result )?
                    // $( @results )?
                    $(
                        $( #[$field_docs:meta] )* $(@)?
                        $field_name:ident: $field_ty:ty
                    ),*
                    $(,)?
                }
            )?
        ),* $(,)?
    ) => {
        impl InstrSequence {
            $(
                #[doc = concat!("Pushes an [`Instruction::", stringify!($name), "`].")]
                ///
                /// Returns the [`Instr`] to query the pushed [`Instruction`].
                pub fn $snake_name(
                    &mut self,
                    $( $( $field_name: impl Into<$field_ty> ),* )?
                ) -> Instr {
                    let pos = Instr(self.instrs.len());
                    self.instrs.push(Instruction::$name {
                        $( $( $field_name: $field_name.into() ),* )?
                    });
                    pos
                }
            )*
        }
    };
}
for_each_op!(define_builder);

/// Iterator yielding the [`Instruction`]s of an [`InstrSequence`].
#[derive(Debug)]
pub struct InstrIter<'a> {
    instrs: slice::Iter<'a, Instruction>,
}

impl<'a> InstrIter<'a> {
    /// Creates a new [`InstrIter`] for the [`InstrSequence`].
    fn new(builder: &'a InstrSequence) -> Self {
        Self {
            instrs: builder.instrs.iter(),
        }
    }
}

impl<'a> Iterator for InstrIter<'a> {
    type Item = &'a Instruction;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.instrs.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.instrs.size_hint()
    }
}

impl<'a> DoubleEndedIterator for InstrIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.instrs.next_back()
    }
}

impl<'a> ExactSizeIterator for InstrIter<'a> {}

/// Iterator yielding the [`Instruction`]s of an [`InstrSequence`] mutably.
#[derive(Debug)]
pub struct InstrIterMut<'a> {
    instrs: slice::IterMut<'a, Instruction>,
}

impl<'a> InstrIterMut<'a> {
    /// Creates a new [`InstrIter`] for the [`InstrSequence`].
    fn new(builder: &'a mut InstrSequence) -> Self {
        Self {
            instrs: builder.instrs.iter_mut(),
        }
    }
}

impl<'a> Iterator for InstrIterMut<'a> {
    type Item = &'a mut Instruction;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.instrs.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.instrs.size_hint()
    }
}

impl<'a> DoubleEndedIterator for InstrIterMut<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.instrs.next_back()
    }
}

impl<'a> ExactSizeIterator for InstrIterMut<'a> {}
