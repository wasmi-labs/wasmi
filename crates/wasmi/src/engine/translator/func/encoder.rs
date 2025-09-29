use super::{Reset, ReusableAllocations};
use crate::{
    core::FuelCostsProvider,
    engine::TranslationError,
    ir::{self, BlockFuel, BranchOffset, Encode as _, Op},
    Engine,
    Error,
};
use alloc::vec::Vec;
use core::{cmp, fmt, marker::PhantomData, mem};

/// A byte position within the encoded byte buffer.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BytePos(usize);

impl From<usize> for BytePos {
    fn from(index: usize) -> Self {
        Self(index)
    }
}

impl From<BytePos> for usize {
    fn from(pos: BytePos) -> Self {
        pos.0
    }
}

/// A position within the encoded byte buffer and its known encoded type.
pub struct Pos<T> {
    /// The underlying byte position.
    value: BytePos,
    /// The type marker denoting what value type has been encoded.
    marker: PhantomData<fn() -> T>,
}

impl<T> From<BytePos> for Pos<T> {
    fn from(value: BytePos) -> Self {
        Self {
            value,
            marker: PhantomData,
        }
    }
}
impl<T> From<Pos<T>> for BytePos {
    fn from(pos: Pos<T>) -> Self {
        pos.value
    }
}
impl<T> Copy for Pos<T> {}
impl<T> Clone for Pos<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            marker: PhantomData,
        }
    }
}
impl<T> PartialEq for Pos<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
impl<T> Eq for Pos<T> {}
impl<T> PartialOrd for Pos<T> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}
impl<T> Ord for Pos<T> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.value.cmp(&other.value)
    }
}
impl<T> fmt::Debug for Pos<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pos")
            .field("value", &self.value)
            .field("marker", &self.marker)
            .finish()
    }
}

#[derive(Debug, Default)]
#[expect(unused)]
pub struct EncodedOps {
    buffer: Vec<u8>,
    temp: Option<(BytePos, TempBuffer)>,
}

/// The kind of temporary/scratch stored object.
#[derive(Debug)]
pub enum TempBuffer {
    /// The temporary object is a [`BranchOffset`].
    BranchOffset(ir::BranchOffset),
    /// The temporary object is a [`BlockFuel`].
    BlockFuel(ir::BlockFuel),
}

impl Reset for EncodedOps {
    fn reset(&mut self) {
        self.buffer.clear();
    }
}

impl EncodedOps {
    /// Returns the next [`BytePos`].
    #[must_use]
    pub fn next_pos(&self) -> BytePos {
        BytePos::from(self.buffer.len())
    }

    /// Takes the temporay buffer if any exists.
    #[must_use]
    pub fn take_temp(&mut self) -> Option<(BytePos, TempBuffer)> {
        self.temp.take()
    }
}

impl ir::Encoder for EncodedOps {
    type Pos = BytePos;
    type Error = TranslationError;

    fn write_bytes(&mut self, bytes: &[u8]) -> Result<Self::Pos, Self::Error> {
        let pos = self.buffer.len();
        if self.buffer.try_reserve(bytes.len()).is_err() {
            return Err(TranslationError::OutOfSystemMemory);
        }
        self.buffer.extend(bytes);
        Ok(BytePos::from(pos))
    }

    fn encode_op_code(&mut self, code: ir::OpCode) -> Result<Self::Pos, Self::Error> {
        encode_op_code(self, code)
    }

    fn branch_offset(
        &mut self,
        pos: Self::Pos,
        branch_offset: BranchOffset,
    ) -> Result<(), Self::Error> {
        debug_assert!(self.temp.is_none());
        self.temp = Some((pos, TempBuffer::BranchOffset(branch_offset)));
        Ok(())
    }

    fn block_fuel(&mut self, pos: Self::Pos, block_fuel: ir::BlockFuel) -> Result<(), Self::Error> {
        debug_assert!(self.temp.is_none());
        self.temp = Some((pos, TempBuffer::BlockFuel(block_fuel)));
        Ok(())
    }
}

/// Creates and encodes the buffer of encoded [`Op`]s for a function.
#[derive(Debug, Default)]
pub struct OpEncoder {
    /// The currently staged [`Op`].
    ///
    /// # Note
    ///
    /// - This allows the last [`Op`] to be peeked, inspected and manipulated.
    /// - For example, this is useful to perform op-code fusion or adjusting the result slot.
    staged: Option<StagedOp>,
    /// The fuel costs of instructions.
    ///
    /// This is `Some` if fuel metering is enabled, otherwise `None`.
    fuel_costs: Option<FuelCostsProvider>,
    /// The list of constructed instructions and their parameters.
    ops: EncodedOps,
}

/// The staged [`Op`] and information about its fuel consumption.
#[derive(Debug, Copy, Clone)]
pub struct StagedOp {
    /// The staged [`Op`].
    op: Op,
    /// Fuel information for the staged [`Op`].
    ///
    /// - The [`Op::ConsumeFuel`] operator associated to the staged [`Op`] if any.
    /// - The fuel required by the staged [`Op`].
    fuel: Option<(Pos<BlockFuel>, BlockFuel)>,
}

impl StagedOp {
    /// Creates a new [`StagedOp`] from `op` and `fuel`.
    pub fn new(op: Op, fuel: Option<(Pos<BlockFuel>, BlockFuel)>) -> Self {
        Self { op, fuel }
    }

    /// Replaces the current [`Op`] with `op`.
    ///
    /// Returns the [`Op`] being replaced.
    pub fn replace(&mut self, op: Op) -> Op {
        mem::replace(&mut self.op, op)
    }
}

impl ReusableAllocations for OpEncoder {
    type Allocations = OpEncoderAllocations;

    fn into_allocations(self) -> Self::Allocations {
        Self::Allocations { ops: self.ops }
    }
}

/// The reusable heap allocations of the [`OpEncoder`].
#[derive(Debug, Default)]
pub struct OpEncoderAllocations {
    /// The list of constructed instructions and their parameters.
    ops: EncodedOps,
}

impl Reset for OpEncoderAllocations {
    fn reset(&mut self) {
        self.ops.reset();
    }
}

impl OpEncoder {
    /// Creates a new [`OpEncoder`].
    pub fn new(engine: &Engine, alloc: OpEncoderAllocations) -> Self {
        let config = engine.config();
        let fuel_costs = config
            .get_consume_fuel()
            .then(|| config.fuel_costs())
            .cloned();
        Self {
            staged: None,
            fuel_costs,
            ops: alloc.ops,
        }
    }

    /// Returns the next allocated [`Pos<Op>`].
    ///
    /// # Panics (Debug)
    ///
    /// If there is a staged [`Op`].
    pub fn next_pos(&self) -> Pos<Op> {
        // TODO: we should probably remove this API again from `OpEncoder`
        debug_assert!(self.staged.is_none());
        Pos::from(self.ops.next_pos())
    }

    /// Returns the staged [`Op`] if any.
    pub fn peek_staged(&self) -> Option<Op> {
        self.staged.map(|staged| staged.op)
    }

    /// Sets the staged [`Op`] to `new_staged` and encodes the previously staged [`Op`] if any.
    ///
    /// Returns the [`Pos<Op>`] of the staged [`Op`] if it was encoded.
    pub fn stage(
        &mut self,
        new_staged: Op,
        fuel_op: Option<Pos<BlockFuel>>,
        fuel_selector: impl FuelCostsSelector,
    ) -> Result<Pos<Op>, Error> {
        let fuel = match (fuel_op, &self.fuel_costs) {
            (None, None) => None,
            (Some(fuel_op), Some(fuel_costs)) => Some((fuel_op, fuel_selector.select(fuel_costs))),
            _ => unreachable!(),
        };
        let new_staged = StagedOp::new(new_staged, fuel);
        if let Some(old_staged) = self.staged.replace(new_staged) {
            self.encode_staged(old_staged)?;
        }
        Ok(Pos::from(self.ops.next_pos()))
    }

    /// Encodes the staged [`Op`] if there is any.
    ///
    /// # Note
    ///
    /// - After this operation there will be no more staged [`Op`].
    /// - Does nothing if there is no staged [`Op`].
    pub fn try_encode_staged(&mut self) -> Result<(), Error> {
        if let Some(staged) = self.staged.take() {
            self.encode_staged(staged)?;
        }
        debug_assert!(self.staged.is_none());
        Ok(())
    }

    /// Encodes the `staged_op`.
    ///
    /// - Bumps fuel consumption of the associated [`Op::ConsumeFuel`] operator.
    /// - Returns the [`Pos<Op>`] of the encoded [`StagedOp`].
    ///
    /// # Panics (Debug)
    ///
    /// If the staged operator unexpectedly issued [`BranchOffset`] or [`BlockFuel`] fields.
    /// Those operators may never be staged and must be taken care of directly.
    fn encode_staged(&mut self, staged: StagedOp) -> Result<Pos<Op>, Error> {
        if let Some((fuel_pos, fuel_used)) = staged.fuel {
            self.bump_fuel_consumption_by(Some(fuel_pos), fuel_used)?;
        }
        let pos = self.encode_impl(staged.op)?;
        debug_assert!(self.ops.temp.is_none());
        Ok(pos)
    }

    /// Drops the staged [`Op`] without encoding it.
    ///
    /// Returns the staged [`Op`]'s fuel information or `None` if fuel metering is disabled.
    ///
    /// # Panics
    ///
    /// If there was no staged [`Op`].
    pub fn drop_staged(&mut self) -> (Option<Pos<BlockFuel>>, BlockFuel) {
        let Some(staged) = self.staged.take() else {
            panic!("could not drop staged `Op` since there was none")
        };
        debug_assert!(self.staged.is_none());
        let fuel_pos = staged.fuel.map(|(pos, _)| pos);
        let fuel_used = staged
            .fuel
            .map(|(_, used)| used)
            .unwrap_or(BlockFuel::from(0));
        (fuel_pos, fuel_used)
    }

    /// Replaces the staged [`Op`] with `new_staged`.
    ///
    /// - This does __not__ encode the currently staged [`Op`] but merely replaces it.
    /// - Returns the [`Pos<Op>`] of the newly staged [`Op`].
    ///
    /// # Panics (Debug)
    ///
    /// If there currently is no staged [`Op`] that can be replaced.
    pub fn replace_staged(&mut self, new_staged: Op) -> Result<Pos<Op>, Error> {
        let Some(staged) = self.staged.as_mut() else {
            panic!("expected a staged `Op` but found `None`")
        };
        staged.replace(new_staged);
        Ok(Pos::from(self.ops.next_pos()))
    }

    /// Encodes an [`Op`] to the [`OpEncoder`] and returns its [`Pos<Op>`].
    ///
    /// # Note
    ///
    /// Bumps the `fuel` of the [`Op::ConsumeFuel`] accordingly.
    pub fn encode(
        &mut self,
        op: Op,
        fuel_pos: Option<Pos<BlockFuel>>,
        fuel_selector: impl FuelCostsSelector,
    ) -> Result<Pos<Op>, Error> {
        self.try_encode_staged()?;
        self.bump_fuel_consumption(fuel_pos, fuel_selector)?;
        let pos = self.encode_impl(op)?;
        debug_assert!(self.ops.take_temp().is_none());
        debug_assert!(self.staged.is_none());
        Ok(pos)
    }

    /// Encodes the given `param` of type `T` to the [`OpEncoder`].
    ///
    /// # Note
    ///
    /// This is usually used to encode parameters of certain variable width [`Op`]s
    /// such as for the encoding of Wasm's `br_table`.
    pub fn encode_param<T>(&mut self, param: T) -> Result<(), Error>
    where
        T: ir::Encode,
    {
        debug_assert!(self.staged.is_none());
        self.encode_impl(param)?;
        debug_assert!(self.ops.take_temp().is_none());
        Ok(())
    }

    /// Encodes an [`Op::ConsumeFuel`] operator to `self`.
    ///
    /// # Note
    ///
    /// The pushed [`Op::ConsumeFuel`] is initialized with base fuel costs.
    pub fn encode_consume_fuel(&mut self) -> Result<Option<Pos<BlockFuel>>, Error> {
        let Some(fuel_costs) = &self.fuel_costs else {
            return Ok(None);
        };
        let consumed_fuel = BlockFuel::from(fuel_costs.base());
        self.try_encode_staged()?;
        Op::consume_fuel(consumed_fuel).encode(&mut self.ops)?;
        let Some((pos, TempBuffer::BlockFuel(_))) = self.ops.take_temp() else {
            unreachable!("expected encoded `BlockFuel` entry but found none")
        };
        debug_assert!(self.staged.is_none());
        Ok(Some(Pos::from(pos)))
    }

    /// Encodes a branch [`Op`] to the [`OpEncoder`] and returns its [`Pos<Op>`] and [`Pos<BranchOffset>`].
    ///
    /// # Note
    ///
    /// Bumps the `fuel` of the [`Op::ConsumeFuel`] accordingly.
    pub fn encode_branch(
        &mut self,
        op: Op,
        fuel_pos: Option<Pos<BlockFuel>>,
        fuel_selector: impl FuelCostsSelector,
    ) -> Result<(Pos<Op>, Pos<BranchOffset>), Error> {
        self.try_encode_staged()?;
        self.bump_fuel_consumption(fuel_pos, fuel_selector)?;
        let pos_op = self.encode_impl(op)?;
        let pos_offset = match self.ops.take_temp() {
            Some((pos, TempBuffer::BranchOffset(_))) => Pos::from(pos),
            _ => panic!("missing encoded position for `BranchOffset`"),
        };
        debug_assert!(self.staged.is_none());
        Ok((pos_op, pos_offset))
    }

    /// Encodes an [`Op`] to the [`OpEncoder`] and returns its [`Pos<Op>`].
    ///
    /// # Note
    ///
    /// - Encodes `last` [`Op`] prior to `op` if `last` is `Some`.
    /// - After this call `last` will yield `None`.
    fn encode_impl<T>(&mut self, op: T) -> Result<Pos<T>, Error>
    where
        T: ir::Encode,
    {
        let pos = self.ops.next_pos();
        op.encode(&mut self.ops)?;
        Ok(Pos::from(pos))
    }

    /// Updates the encoded [`BranchOffset`] at `pos` to `offset`.
    ///
    /// # Panics
    ///
    /// - If `pos` was out of bounds for `self`.
    /// - If the [`BranchOffset`] at `pos` failed to be decoded, updated or re-encoded.
    pub fn update_branch_offset(
        &mut self,
        pos: Pos<BranchOffset>,
        offset: BranchOffset,
    ) -> Result<(), Error> {
        self.update_encoded(pos, |_| Some(offset));
        Ok(())
    }

    /// Bumps consumed fuel for [`Op::ConsumeFuel`] at `fuel_pos` by `fuel_selector(fuel_costs)`.
    ///
    /// Does nothing if fuel metering is disabled.
    ///
    /// # Errors
    ///
    /// If consumed fuel is out of bounds after this operation.
    fn bump_fuel_consumption(
        &mut self,
        fuel_pos: Option<Pos<BlockFuel>>,
        fuel_selector: impl FuelCostsSelector,
    ) -> Result<(), Error> {
        debug_assert_eq!(fuel_pos.is_some(), self.fuel_costs.is_some());
        let fuel_used = match &self.fuel_costs {
            None => return Ok(()),
            Some(fuel_costs) => fuel_selector.select(fuel_costs),
        };
        self.bump_fuel_consumption_by(fuel_pos, fuel_used)
    }

    /// Bumps consumed fuel for [`Op::ConsumeFuel`] at `fuel_pos` by `delta`.
    ///
    /// Does nothing if fuel metering is disabled.
    ///
    /// # Errors
    ///
    /// If consumed fuel is out of bounds after this operation.
    fn bump_fuel_consumption_by(
        &mut self,
        fuel_pos: Option<Pos<BlockFuel>>,
        delta: BlockFuel,
    ) -> Result<(), Error> {
        debug_assert_eq!(fuel_pos.is_some(), self.fuel_costs.is_some());
        let fuel_pos = match fuel_pos {
            None => return Ok(()),
            Some(fuel_pos) => fuel_pos,
        };
        self.update_encoded(fuel_pos, |mut fuel| -> Option<BlockFuel> {
            fuel.bump_by(u64::from(delta)).ok()?;
            Some(fuel)
        });
        Ok(())
    }

    /// Returns an iterator yielding all encoded [`Op`]s of the [`OpEncoder`] as bytes.
    pub fn encoded_ops(&mut self) -> &[u8] {
        debug_assert!(self.staged.is_none());
        &self.ops.buffer[..]
    }
}

/// Error indicating that in-place updating of encoded items failed.
struct UpdateEncodedError<T> {
    /// The underlying kind of error.
    kind: UpdateEncodedErrorKind,
    /// The type that is decoded, updated and re-encoded.
    marker: PhantomData<fn() -> T>,
}

impl<T> From<UpdateEncodedErrorKind> for UpdateEncodedError<T> {
    fn from(kind: UpdateEncodedErrorKind) -> Self {
        Self {
            kind,
            marker: PhantomData,
        }
    }
}
impl<T> Clone for UpdateEncodedError<T> {
    fn clone(&self) -> Self {
        Self {
            kind: self.kind,
            marker: PhantomData,
        }
    }
}
impl<T> Copy for UpdateEncodedError<T> {}
impl<T> fmt::Debug for UpdateEncodedError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UpdateEncodedError")
            .field("kind", &self.kind)
            .field("marker", &self.marker)
            .finish()
    }
}
impl<T> fmt::Display for UpdateEncodedError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self.kind {
            UpdateEncodedErrorKind::BufferOutOfBounds => "buffer out of bounds",
            UpdateEncodedErrorKind::FailedToDecode => "failed to decode",
            UpdateEncodedErrorKind::FailedToEncode => "failed to encode",
            UpdateEncodedErrorKind::FailedToUpdateEncoded => "failed to update encoded",
        };
        let type_name = core::any::type_name::<T>();
        write!(f, "{message}: {type_name}")
    }
}

/// Kinds of errors indicating that in-place updating of encoded items failed.
#[derive(Debug, Copy, Clone)]
enum UpdateEncodedErrorKind {
    /// Buffer is out of bounds for the position of the update.
    BufferOutOfBounds,
    /// Failed to decode the encoded item.
    FailedToDecode,
    /// Failed to encode the updated item.
    FailedToEncode,
    /// Failed to update the encoded item.
    FailedToUpdateEncoded,
}

impl OpEncoder {
    /// Updates an encoded value `v` of type `T` at `pos` in-place using the result of `f(v)`.
    ///
    /// # Panics
    ///
    /// - If the underlying bytes buffer is out of bounds for `pos`.
    /// - If decodiing of `T` at `pos` fails.
    /// - If encodiing of `T` at `pos` fails.
    fn update_encoded<T>(&mut self, pos: Pos<T>, f: impl FnOnce(T) -> Option<T>)
    where
        T: ir::Encode + ir::Decode,
    {
        if let Err(error) = self
            .update_encoded_or_err(pos, f)
            .map_err(<UpdateEncodedError<T>>::from)
        {
            panic!("`OpEncoder::update_encoded` unexpectedly failed: {error}")
        }
    }

    /// Updates a value of type `T` at `pos` using `f` in the encoded buffer.
    ///
    /// # Errors
    ///
    /// - If the underlying bytes buffer is out of bounds for `pos`.
    /// - If decodiing of `T` at `pos` fails.
    /// - If encodiing of `T` at `pos` fails.
    /// - If `f(value)` returns `None` and thus updating failed.
    fn update_encoded_or_err<T>(
        &mut self,
        pos: Pos<T>,
        f: impl FnOnce(T) -> Option<T>,
    ) -> Result<(), UpdateEncodedErrorKind>
    where
        T: ir::Decode + ir::Encode,
    {
        let at = usize::from(BytePos::from(pos));
        let Some(buffer) = self.ops.buffer.get_mut(at..) else {
            return Err(UpdateEncodedErrorKind::BufferOutOfBounds);
        };
        let Ok(decoded) = T::decode(&mut &buffer[..]) else {
            return Err(UpdateEncodedErrorKind::FailedToDecode);
        };
        let Some(updated) = f(decoded) else {
            return Err(UpdateEncodedErrorKind::FailedToUpdateEncoded);
        };
        if updated.encode(&mut SliceEncoder::from(buffer)).is_err() {
            return Err(UpdateEncodedErrorKind::FailedToEncode);
        }
        Ok(())
    }
}

/// Utility type to encode items to a slice of bytes.
pub struct SliceEncoder<'a> {
    /// The underlying bytes that will store the encoded items.
    bytes: &'a mut [u8],
}

/// An error that may occur upon encoding items to a byte slice.
#[derive(Debug, Copy, Clone)]
pub struct SliceEncoderError;

impl<'a> From<&'a mut [u8]> for SliceEncoder<'a> {
    fn from(bytes: &'a mut [u8]) -> Self {
        Self { bytes }
    }
}

impl<'a> ir::Encoder for SliceEncoder<'a> {
    type Pos = ();
    type Error = SliceEncoderError;

    fn write_bytes(&mut self, bytes: &[u8]) -> Result<Self::Pos, Self::Error> {
        let Some(buffer) = self.bytes.get_mut(..bytes.len()) else {
            return Err(SliceEncoderError);
        };
        buffer.copy_from_slice(bytes);
        Ok(())
    }

    fn encode_op_code(&mut self, code: ir::OpCode) -> Result<Self::Pos, Self::Error> {
        encode_op_code(self, code)
    }

    fn branch_offset(
        &mut self,
        _pos: Self::Pos,
        _branch_offset: BranchOffset,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn block_fuel(&mut self, _pos: Self::Pos, _block_fuel: BlockFuel) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// Convenience trait to wrap type usable as fuel costs selectors.
pub trait FuelCostsSelector {
    /// Selects the fuel usage from the [`FuelCostsProvider`].
    fn select(self, costs: &FuelCostsProvider) -> BlockFuel;
}

impl<T> FuelCostsSelector for T
where
    T: FnOnce(&FuelCostsProvider) -> u64,
{
    fn select(self, costs: &FuelCostsProvider) -> BlockFuel {
        BlockFuel::from(self(costs))
    }
}

impl FuelCostsSelector for BlockFuel {
    fn select(self, _costs: &FuelCostsProvider) -> BlockFuel {
        self
    }
}

/// Encodes an [`ir::OpCode`] to a generic [`ir::Encoder`].
fn encode_op_code<E: ir::Encoder>(encoder: &mut E, code: ir::OpCode) -> Result<E::Pos, E::Error> {
    // Note: this implements encoding for indirect threading.
    //
    // For direct threading we need to know ahead of time about the
    // function pointers of all operator execution handlers which
    // are defined in the Wasmi executor and available to the translator.
    u16::from(code).encode(encoder)
}
