use crate::{
    decode::CheckedOpDecoderMut,
    for_each_newtype,
    for_each_op,
    CheckedOpDecoder,
    Op,
    OpMut,
    Visitor,
};
use ::core::{fmt, iter, mem, mem::MaybeUninit, slice};

/// A byte stream encoder.
///
/// Efficiently encodes items into their generic byte representation.
#[derive(Debug, Default)]
pub struct Encoder {
    /// The bytes of instructions encoded to the [`Encoder`].
    bytes: Vec<u8>,
}

impl Encoder {
    /// Returns a shared reference to the underlying encoded bytes of the [`Encoder`].
    pub fn as_slice(&self) -> &[u8] {
        &self.bytes[..]
    }

    /// Returns a mutable reference to the underlying encoded bytes of the [`Encoder`].
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        &mut self.bytes[..]
    }

    /// Returns the number of bytes for all encoded instructions in the [`Encoder`].
    pub fn len_bytes(&self) -> usize {
        self.bytes.len()
    }
}

/// Trait implemented by types that can encode their instances into a byte represenation.
pub trait Encode: EncodeSizeHint {
    /// Encodes `self` via the `encoder` into its byte representation.
    fn encode<T>(&self, encoder: &mut T)
    where
        T: Extend<u8>;
}

/// Used to query the number of bytes requires to encode instances of `Self`.
pub trait EncodeSizeHint {
    /// The number of bytes required to encode `self`.
    fn size_hint(&self) -> usize;
}

impl<T> EncodeSizeHint for T
where
    T: ExactSizeEncoding,
{
    fn size_hint(&self) -> usize {
        encoding_size_of::<Self>()
    }
}

/// Marker trait implemented by types which have a fixed encoding size equal to `size_of<Self>`.
pub trait ExactSizeEncoding: Encode + Sized + Copy {}

/// Returns the number of bytes `T` requires for its exact size encoding.
///
/// # Note
///
/// This is a convenience method and always returns the same value as `size_of::<T>()`.
pub const fn encoding_size_of<T: ExactSizeEncoding>() -> usize {
    mem::size_of::<T>()
}

macro_rules! impl_exact_encoding_size_for {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl ExactSizeEncoding for $ty {}
        )*
    }
}
impl_exact_encoding_size_for!(
    bool,
    i8,
    i16,
    i32,
    i64,
    i128,
    u8,
    u16,
    u32,
    u64,
    u128,
    f32,
    f64,
    crate::OpCode,
    crate::Sign,
    crate::RegSpan,
    crate::BranchTableTarget,
);

/// Trait implemented by byte encoders.
pub trait Extend<T> {
    fn extend<I>(&mut self, items: I)
    where
        I: IntoIterator<Item = T>;
}

impl Extend<u8> for Encoder {
    fn extend<I>(&mut self, items: I)
    where
        I: IntoIterator<Item = u8>,
    {
        let bytes = items.into_iter();
        self.bytes.extend(bytes);
    }
}

/// Thin-wrapper around a byte slice to encode until the slice is fully encoded.
#[derive(Debug)]
pub struct SliceEncoder<'a> {
    /// The byte slice to encode into.
    slice: &'a mut [u8],
    /// The position of the last encoded byte in `slice.`
    pos: usize,
}

impl<'a> From<&'a mut [u8]> for SliceEncoder<'a> {
    fn from(slice: &'a mut [u8]) -> Self {
        Self { slice, pos: 0 }
    }
}

impl SliceEncoder<'_> {
    /// Returns the number of unencoded bytes within `self`.
    pub fn len_unencoded(&self) -> usize {
        self.slice.len() - self.pos
    }

    /// Returns `true` if there are still some unencoded bytes left in `self`.
    pub fn has_unencoded(&self) -> bool {
        self.len_unencoded() != 0
    }
}

impl<'a> Extend<u8> for SliceEncoder<'a> {
    fn extend<I>(&mut self, items: I)
    where
        I: IntoIterator<Item = u8>,
    {
        for byte in items {
            self.slice[self.pos] = byte;
            self.pos += 1;
        }
    }
}

macro_rules! impl_encode_as_byte {
    ( $( $ty:ty ),* ) => {
        $(
            impl Encode for $ty {
                fn encode<T>(&self, encoder: &mut T)
                where
                    T: Extend<u8>,
                {
                    encoder.extend(iter::once(*self as _))
                }
            }
        )*
    };
}
impl_encode_as_byte!(bool, i8, u8);

macro_rules! impl_encode_for_primitive {
    ( $( $ty:ty ),* ) => {
        $(
            impl Encode for $ty {
                fn encode<T>(&self, encoder: &mut T)
                where
                    T: Extend<u8>,
                {
                    encoder.extend(self.to_ne_bytes())
                }
            }
        )*
    };
}
impl_encode_for_primitive!(i16, u16, i32, u32, i64, u64, i128, u128, f32, f64);

macro_rules! impl_encode_for_nonzero {
    ( $( $ty:ty ),* $(,)? ) => {
        $(
            impl Encode for ::core::num::NonZero<$ty> {
                fn encode<T>(&self, encoder: &mut T)
                where
                    T: Extend<u8>,
                {
                    self.get().encode(encoder)
                }
            }

            impl ExactSizeEncoding for ::core::num::NonZero<$ty> {}
        )*
    };
}
impl_encode_for_nonzero!(i8, u8, i16, u16, i32, u32, i64, u64, i128, u128);

macro_rules! impl_encode_for_newtype {
    (
        $(
            $( #[$docs:meta] )*
            struct $name:ident($vis:vis $ty:ty);
        )*
    ) => {
        $(
            impl Encode for crate::$name {
                fn encode<T>(&self, encoder: &mut T)
                where
                    T: Extend<u8>,
                {
                    self.0.encode(encoder);
                }
            }

            impl ExactSizeEncoding for crate::$name {}
        )*
    };
}
for_each_newtype!(impl_encode_for_newtype);

impl Encode for crate::OpCode {
    fn encode<T>(&self, encoder: &mut T)
    where
        T: Extend<u8>,
    {
        (*self as u16).encode(encoder)
    }
}

impl Encode for crate::Sign {
    fn encode<T>(&self, encoder: &mut T)
    where
        T: Extend<u8>,
    {
        (*self as u8).encode(encoder)
    }
}

impl Encode for crate::RegSpan {
    fn encode<T>(&self, encoder: &mut T)
    where
        T: Extend<u8>,
    {
        self.head().encode(encoder)
    }
}

impl Encode for crate::BranchTableTarget {
    fn encode<T>(&self, encoder: &mut T)
    where
        T: Extend<u8>,
    {
        self.value.encode(encoder)
    }
}

impl<T: Copy> Encode for crate::Unalign<T>
where
    T: ExactSizeEncoding,
{
    fn encode<E>(&self, encoder: &mut E)
    where
        E: Extend<u8>,
    {
        self.get().encode(encoder)
    }
}

impl<T> ExactSizeEncoding for crate::Unalign<T> where T: ExactSizeEncoding {}

impl<'a, T> Encode for crate::Slice<'a, T>
where
    T: Copy + Encode + ExactSizeEncoding,
{
    fn encode<E>(&self, encoder: &mut E)
    where
        E: Extend<u8>,
    {
        self.len().encode(encoder);
        for item in self.as_ref() {
            item.encode(encoder);
        }
    }
}

impl<'a, T> EncodeSizeHint for crate::Slice<'a, T>
where
    T: Copy + ExactSizeEncoding,
{
    fn size_hint(&self) -> usize {
        encoding_size_of::<u16>()
            .saturating_add(usize::from(self.len).saturating_mul(encoding_size_of::<T>()))
    }
}

impl Encode for crate::TrapCode {
    fn encode<T>(&self, encoder: &mut T)
    where
        T: Extend<u8>,
    {
        (*self as u8).encode(encoder)
    }
}

impl ExactSizeEncoding for crate::TrapCode {}

macro_rules! define_encode_for_op {
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
        impl<'op> Encode for crate::Op<'op> {
            fn encode<T>(&self, __encoder: &mut T)
            where
                T: Extend<u8>,
            {
                match self {
                    $(
                        Self::$camel_name(__op) => {
                            crate::OpCode::$camel_name.encode(__encoder);
                            __op.encode(__encoder);
                        }
                    )*
                }
            }
        }

        impl<'op> EncodeSizeHint for crate::Op<'op> {
            fn size_hint(&self) -> usize {
                match self {
                    $(
                        Self::$camel_name(__op) => {
                            crate::OpCode::$camel_name.size_hint().wrapping_add(
                                <crate::op::$camel_name as EncodeSizeHint>::size_hint(__op)
                            )
                        },
                    )*
                }
            }
        }

        $(
            impl$(<$lt>)? Encode for crate::op::$camel_name $(<$lt>)? {
                fn encode<T>(&self, __encoder: &mut T)
                where
                    T: Extend<u8>,
                {
                    $(
                        $(
                            self.$field_ident.encode(__encoder)
                        );*
                    )?
                }
            }

            impl$(<$lt>)? EncodeSizeHint for crate::op::$camel_name $(<$lt>)? {
                fn size_hint(&self) -> usize {
                    0_usize
                    $(
                        $( .wrapping_add(self.$field_ident.size_hint()) )*
                    )?
                }
            }
        )*
    };
}
for_each_op!(define_encode_for_op);

/// An [`Op`] encoder.
#[derive(Debug, Default)]
pub struct OpEncoder {
    /// The underlying encoder.
    encoder: Encoder,
    /// The end indices of the encoded [`Op`].
    ///
    /// The length of `ends` equals the number of encoded [`Op`] in the [`OpEncoder`].
    ends: Vec<OpPos>,
}

/// A position denoting the existence of an encoded [`Op`] in an [`OpEncoder`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OpPos(usize);

impl From<OpPos> for usize {
    fn from(value: OpPos) -> Self {
        value.0
    }
}

impl OpEncoder {
    /// Encodes `op` and pushes the encoded `op` to `self`.
    pub fn push<'op>(&mut self, op: impl Into<Op<'op>>) -> OpPos {
        self.push_op(op.into())
    }

    /// Encodes `op` and pushes the encoded `op` to `self`.
    fn push_op(&mut self, op: Op) -> OpPos {
        op.encode(&mut self.encoder);
        let pos = OpPos(self.encoder.len_bytes());
        self.ends.push(pos);
        pos
    }

    /// Pops the last encoded [`Op`] from `self`.
    ///
    /// Returns `None` if `self` is empty.
    ///
    /// # Note
    ///
    /// Prefer this over [`Self::pop_decode`] and [`Self::pop_visit`] if you are not
    /// interested in the popped [`Op`] since this is likely much more efficient.
    pub fn pop_drop(&mut self) -> Option<()> {
        self.pop_impl(drop)
    }

    /// Pops the last encoded [`Op`] from `self` and returns it decoded.
    ///
    /// Returns `None` if `self` is empty.
    ///
    /// # Panics
    ///
    /// If decoding of the popped [`Op`] fails.
    pub fn pop_decode(&mut self) -> Option<Op> {
        self.pop_impl(|mut decoder| {
            decoder.decode().unwrap_or_else(|error| {
                panic!("OpEncoder::pop_decode: failed to decode `Op`: {error}")
            })
        })
    }

    /// Pops the last encoded [`Op`] and calls the associateed `visitor` method.
    ///
    /// Returns the result of the `visitor` method call.
    /// After this operation `self` no longer contains the popped [`Op`].
    ///
    /// # Panics
    ///
    /// If decoding of the popped [`Op`] fails.
    pub fn pop_visit<V>(&mut self, visitor: &mut V) -> Option<V::Output>
    where
        V: Visitor,
    {
        self.pop_impl::<V::Output>(|mut decoder| {
            decoder.visit(visitor).unwrap_or_else(|error| {
                panic!("`OpEncoder::pop_visit`: failed to decode the popped `Op`: {error}")
            })
        })
    }

    /// Implementation detail for [`Self::pop_drop`], [`Self::pop_decode`] and [`Self::pop_visit`].
    fn pop_impl<'a, R>(&'a mut self, f: impl FnOnce(CheckedOpDecoder<'a>) -> R) -> Option<R> {
        let last_pos = self.last_pos()?;
        let (start, end) = self
            .get_start_end(last_pos)
            .expect("must have `start` and `end` for `OpPos` returned by `last_pos`");
        let len_encoded = end - start;
        debug_assert_eq!(end, self.as_bytes().len());
        // Safety: this is safe since `Op` is `Copy` and thus nothing needs to be dropped.
        unsafe { self.encoder.bytes.set_len(start) };
        let bytes = &self.encoder.bytes.spare_capacity_mut()[..len_encoded];
        // Safety: this is safe as this exact slice of bytes have already been initialized
        //         and are just marked as uninitialized since we truncated the `Vec`'s length above.
        let bytes: &[u8] = unsafe { &*(bytes as *const [MaybeUninit<u8>] as *const [u8]) };
        let result = f(CheckedOpDecoder::new(bytes));
        self.ends
            .pop()
            .expect("must have an `ends` item for every `Op`");
        Some(result)
    }

    /// Returns the n-th encoded [`Op`] in `self` if any.
    ///
    /// Returns `None` if there is no encoded [`Op`] at `pos`.
    ///
    /// # Panics
    ///
    /// If decoding of the [`Op`] at `pos` fails.
    pub fn get(&self, pos: OpPos) -> Option<Op> {
        let bytes = self.get_bytes(pos)?;
        let Ok(decoded) = CheckedOpDecoder::new(bytes).decode() else {
            panic!("`OpEncoder::get`: failed to decode `Op` at: {pos:?}")
        };
        Some(decoded)
    }

    /// Returns the n-th encoded [`OpMut`] in `self` if any.
    ///
    /// Returns `None` if there is no encoded [`OpMut`] at `pos`.
    ///
    /// # Panics
    ///
    /// If decoding of the [`OpMut`] at `pos` fails.
    pub fn get_mut(&mut self, pos: OpPos) -> Option<OpMut> {
        let bytes = self.get_bytes_mut(pos)?;
        let Ok(decoded) = CheckedOpDecoderMut::new(bytes).decode_mut() else {
            panic!("`OpEncoder::get_mut`: failed to decode `Op` at: {pos:?}")
        };
        Some(decoded)
    }

    /// Returns the bytes of the encoded [`Op`] in `self`.
    pub fn as_bytes(&self) -> &[u8] {
        self.encoder.as_slice()
    }

    /// Returns the start and end indices within the encoded byte stream for `pos`.
    fn get_start_end(&self, pos: OpPos) -> Option<(usize, usize)> {
        let pos = pos.0;
        let end = self.ends.get(pos)?.0;
        let start = self
            .ends
            .get(pos.wrapping_sub(1))
            .copied()
            .map(usize::from)
            .unwrap_or(0);
        Some((start, end))
    }

    /// Returns the bytes of the encoded [`Op`] associated to `pos` if any.
    fn get_bytes(&self, pos: OpPos) -> Option<&[u8]> {
        let (start, end) = self.get_start_end(pos)?;
        let bytes = &self.encoder.as_slice()[start..end];
        Some(bytes)
    }

    /// Returns the bytes of the encoded [`Op`] associated to `pos` if any.
    fn get_bytes_mut(&mut self, pos: OpPos) -> Option<&mut [u8]> {
        let (start, end) = self.get_start_end(pos)?;
        let bytes = &mut self.encoder.as_slice_mut()[start..end];
        Some(bytes)
    }

    /// Returns the last `OpPos`; or returns `None` if `self` is empty.
    fn last_pos(&self) -> Option<OpPos> {
        if self.is_empty() {
            return None;
        }
        Some(OpPos(self.ends.len() - 1))
    }

    /// Returns the last two `OpPos`; or returns `None` if the length of `self` is less than 2.
    fn last_pos_2(&self) -> Option<(OpPos, OpPos)> {
        if self.len() < 2 {
            return None;
        }
        let len = self.ends.len();
        Some((OpPos(len - 2), OpPos(len - 1)))
    }

    /// Returns the last encoded [`Op`] in `self` if any.
    ///
    /// Returns `None` if `self` is empty.
    ///
    /// # Panics
    ///
    /// If decoding of the last encoded [`Op`] fails.
    pub fn last(&self) -> Option<Op> {
        let last_pos = self.last_pos()?;
        self.get(last_pos)
    }

    /// Returns the two last encoded [`Op`] in `self` if any.
    ///
    /// Returns `None` if `self` is empty.
    ///
    /// # Panics
    ///
    /// If decoding of the last encoded [`Op`] fails.
    pub fn last_2(&self) -> Option<(Op, Op)> {
        let (p0, p1) = self.last_pos_2()?;
        let op0 = self.get(p0)?;
        let op1 = self.get(p1)?;
        Some((op0, op1))
    }

    /// Returns the number of encoded [`Op`] in `self`.
    pub fn len(&self) -> usize {
        self.ends.len()
    }

    /// Returns `true` if `self` contains no encoded [`Op`].
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Visits the n-th encoded [`Op`] in `self` if any by `visitor`.
    ///
    /// - Returns the value returned by `visitor` if visitation succeeded.
    /// - Returns `None` if there is no encoded [`Op`] at `pos`.
    ///
    /// # Panics
    ///
    /// If decoding of the [`Op`] at `pos` fails.
    pub fn visit<V: Visitor>(&self, pos: OpPos, visitor: &mut V) -> Option<V::Output> {
        let bytes = self.get_bytes(pos)?;
        let Ok(result) = CheckedOpDecoder::new(bytes).visit(visitor) else {
            panic!("`OpEncoder::get`: failed to decode `Op` at: {pos:?}")
        };
        Some(result)
    }

    /// Patches the [`Op`] at `pos` to be `new_op`.
    ///
    /// # Errors
    ///
    /// - If `pos` is invalid for `self`.
    /// - If `new_op` has a different encoding size from the to-be patched [`Op`].
    pub fn patch<'a>(&mut self, pos: OpPos, new_op: impl Into<Op<'a>>) -> Result<(), PatchError> {
        self.patch_impl(pos, new_op.into())
    }

    /// Implementation details of [`Self::patch`].
    fn patch_impl(&mut self, pos: OpPos, new_op: Op) -> Result<(), PatchError> {
        let Some(bytes) = self.get_bytes_mut(pos) else {
            return Err(PatchError::InvalidOpPos);
        };
        let size_hint = new_op.size_hint();
        if bytes.len() != size_hint {
            return Err(PatchError::EncodedSizeMismatch {
                old_size: bytes.len(),
                new_size: size_hint,
            });
        }
        let mut encoder = SliceEncoder::from(bytes);
        new_op.encode(&mut encoder);
        assert!(
            !encoder.has_unencoded(),
            "unexpected mismatch in encoding size between old `Op` and new `Op`",
        );
        Ok(())
    }

    /// Returns an iterator over all [`Op`] encoded by `self`.
    pub fn iter(&self) -> OpIter {
        OpIter::new(self)
    }

    /// Returns an iterator over all [`OpMut`] encoded by `self`.
    pub fn iter_mut(&mut self) -> OpIterMut {
        OpIterMut::new(self)
    }
}

impl<'a> IntoIterator for &'a OpEncoder {
    type Item = Op<'a>;
    type IntoIter = OpIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut OpEncoder {
    type Item = OpMut<'a>;
    type IntoIter = OpIterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// Error that may be returned by [`OpEncoder::patch`].
#[derive(Debug)]
pub enum PatchError {
    /// Encountered when trying to patch an [`Op`] at an invalid [`OpPos`].
    InvalidOpPos,
    /// Encountered when trying to patch an [`Op`] with one with a differing encoding size.
    EncodedSizeMismatch {
        /// The encoding size of the to-be patched [`Op`].
        old_size: usize,
        /// The encoding size of the new [`Op`].
        new_size: usize,
    },
}

impl fmt::Display for PatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOpPos => {
                write!(f, "encountered invalid `OpPos` for patching")
            }
            Self::EncodedSizeMismatch { old_size, new_size } => {
                write!(f, "new `Op` required {new_size} bytes for its encoding but old `Op` required {old_size} bytes")
            }
        }
    }
}

/// An iterator over the [`Op`]s of an [`OpEncoder`].
#[derive(Debug, Clone)]
pub struct OpIter<'a> {
    /// The underlying encoded bytes of all `Op`.
    bytes: &'a [u8],
    /// The end indices of all `Op`.`
    ends: slice::Iter<'a, OpPos>,
    /// The current start index of the iterator.
    start: usize,
}

impl<'a> OpIter<'a> {
    /// Create a new [`OpIter`] from the given [`OpEncoder`].
    fn new(encoder: &'a OpEncoder) -> Self {
        Self {
            bytes: encoder.as_bytes(),
            ends: encoder.ends.iter(),
            start: 0,
        }
    }
}

impl<'a> Iterator for OpIter<'a> {
    type Item = Op<'a>;

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.ends.size_hint()
    }

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.start;
        let end = self.ends.next()?.0;
        self.start = end;
        let bytes = &self.bytes[start..end];
        let op = CheckedOpDecoder::new(bytes)
            .decode()
            .unwrap_or_else(|error| {
                panic!("expect all `Op` in `OpEncoder` to be valid but encountered: {error}")
            });
        Some(op)
    }
}

impl<'a> ExactSizeIterator for OpIter<'a> {}

/// An iterator over the [`Op`]s of an [`OpEncoder`].
#[derive(Debug)]
pub struct OpIterMut<'a> {
    /// The underlying encoded bytes of all `Op`.
    bytes: &'a mut [u8],
    /// The end indices of all `Op`.`
    ends: slice::Iter<'a, OpPos>,
    /// The current start index of the iterator.
    start: usize,
}

impl<'a> OpIterMut<'a> {
    /// Create a new [`OpIterMut`] from the given [`OpEncoder`].
    fn new(encoder: &'a mut OpEncoder) -> Self {
        Self {
            bytes: encoder.encoder.as_slice_mut(),
            ends: encoder.ends.iter(),
            start: 0,
        }
    }
}

impl<'a> Iterator for OpIterMut<'a> {
    type Item = OpMut<'a>;

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.ends.size_hint()
    }

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.start;
        let end = self.ends.next()?.0;
        self.start = end;
        // Safety: it is safe to extend the lifetime of `self.bytes` to `'op` because:
        //
        // - The `op` lifetime is their actual lifetime
        // - Shared mutable access is avoided since bytes of different encoded `Op`
        //   do never overlap and thus no bytes will ever be mutably shared via this iterator.
        // - It is not possible to have multiple `OpIterMut` for the same `OpEncoder` at the same time.
        // - `OpIterMut` cannot be cloned.
        let bytes = unsafe { crate::decode::extend_lifetime(&mut self.bytes[start..end]) };
        let op = CheckedOpDecoderMut::new(bytes)
            .decode_mut()
            .unwrap_or_else(|error| {
                panic!("expect all `Op` in `OpEncoder` to be valid but encountered: {error}")
            });
        Some(op)
    }
}

impl<'a> ExactSizeIterator for OpIterMut<'a> {}
