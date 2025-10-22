use super::{Reset, ReusableAllocations};
use crate::{
    core::FuelCostsProvider,
    engine::{
        executor::op_code_to_handler,
        translator::{
            comparator::UpdateBranchOffset,
            func::{
                labels::{Label, ResolvedLabelUser},
                LabelRef,
                LabelRegistry,
            },
        },
        TranslationError,
    },
    ir::{self, BlockFuel, BranchOffset, Encode as _, Op},
    Engine,
    Error,
};
use alloc::vec::Vec;
use core::{cmp, fmt, marker::PhantomData, mem};

/// Fuel amount required by certain operators.
type FuelUsed = u64;

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
        *self
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
        Some(self.cmp(other))
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
pub struct EncodedOps {
    buffer: Vec<u8>,
    temp: Option<ReportingPos>,
}

/// A [`Pos`] of an encoded item that needs to be reported back.
#[derive(Debug)]
enum ReportingPos {
    /// The temporary object is a [`BranchOffset`].
    BranchOffset(Pos<BranchOffset>),
    /// The temporary object is a [`BlockFuel`].
    BlockFuel(Pos<BlockFuel>),
}

impl Reset for EncodedOps {
    fn reset(&mut self) {
        self.buffer.clear();
    }
}

impl EncodedOps {
    /// Returns the next [`BytePos`].
    #[must_use]
    fn next_pos(&self) -> BytePos {
        BytePos::from(self.buffer.len())
    }

    /// Takes the reporting [`Pos`] if any exists.
    #[must_use]
    fn take_reporting_pos(&mut self) -> Option<ReportingPos> {
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
        _branch_offset: BranchOffset,
    ) -> Result<(), Self::Error> {
        debug_assert!(self.temp.is_none());
        self.temp = Some(ReportingPos::BranchOffset(pos.into()));
        Ok(())
    }

    fn block_fuel(
        &mut self,
        pos: Self::Pos,
        _block_fuel: ir::BlockFuel,
    ) -> Result<(), Self::Error> {
        debug_assert!(self.temp.is_none());
        self.temp = Some(ReportingPos::BlockFuel(pos.into()));
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
    /// Labels and label users for control flow and encoded branch operators.
    labels: LabelRegistry,
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
    fuel: Option<(Pos<BlockFuel>, FuelUsed)>,
}

impl StagedOp {
    /// Creates a new [`StagedOp`] from `op` and `fuel`.
    pub fn new(op: Op, fuel: Option<(Pos<BlockFuel>, FuelUsed)>) -> Self {
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
        Self::Allocations {
            ops: self.ops,
            labels: self.labels,
        }
    }
}

/// The reusable heap allocations of the [`OpEncoder`].
#[derive(Debug, Default)]
pub struct OpEncoderAllocations {
    /// The list of constructed instructions and their parameters.
    ops: EncodedOps,
    /// Labels and label users for control flow and encoded branch operators.
    labels: LabelRegistry,
}

impl Reset for OpEncoderAllocations {
    fn reset(&mut self) {
        self.ops.reset();
        self.labels.reset();
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
            labels: alloc.labels,
        }
    }

    /// Allocates a new unpinned [`Label`].
    pub fn new_label(&mut self) -> LabelRef {
        self.labels.new_label()
    }

    /// Pins the [`Label`] at `lref` to the current encoded bytestream position.
    ///
    /// # Panics
    ///
    /// If there is a staged [`Op`].
    pub fn pin_label(&mut self, lref: LabelRef) -> Result<(), Error> {
        self.try_encode_staged()?;
        let next_pos = Pos::from(self.ops.next_pos());
        self.labels.pin_label(lref, next_pos);
        Ok(())
    }

    /// Pins the [`Label`] at `lref` to the current encoded bytestream position if unpinned.
    ///
    /// # Note
    ///
    /// Does nothing if the label is already pinned.
    ///
    /// # Panics
    ///
    /// If there is a staged [`Op`].
    pub fn pin_label_if_unpinned(&mut self, lref: LabelRef) -> Result<(), Error> {
        self.try_encode_staged()?;
        let next_pos = Pos::from(self.ops.next_pos());
        self.labels.pin_label_if_unpinned(lref, next_pos);
        Ok(())
    }

    /// Resolves the [`BranchOffset`] to `lref` from the current encoded bytestream position if `lref` is pinned.
    ///
    ///
    /// # Note
    ///
    /// Returns an uninitialized [`BranchOffset`] if `lref` refers to an unpinned [`Label`].
    ///
    /// # Panics
    ///
    /// If there is a staged [`Op`].
    fn try_resolve_label(&mut self, lref: LabelRef) -> Result<BranchOffset, Error> {
        assert!(self.staged.is_none());
        let src = self.ops.next_pos();
        let offset = match self.labels.get_label(lref) {
            Label::Pinned(dst) => trace_branch_offset(src, dst)?,
            Label::Unpinned => BranchOffset::uninit(),
        };
        Ok(offset)
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
    pub fn drop_staged(&mut self) -> (Option<Pos<BlockFuel>>, FuelUsed) {
        let Some(staged) = self.staged.take() else {
            panic!("could not drop staged `Op` since there was none")
        };
        debug_assert!(self.staged.is_none());
        let fuel_pos = staged.fuel.map(|(pos, _)| pos);
        let fuel_used = staged.fuel.map(|(_, used)| used).unwrap_or(0);
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

    /// Encodes an item of type `T` to the [`OpEncoder`] and returns its [`Pos`].
    ///
    /// # Note
    ///
    /// Bumps the `fuel` of the [`Op::ConsumeFuel`] accordingly.
    pub fn encode<T: ir::Encode>(
        &mut self,
        op: T,
        fuel_pos: Option<Pos<BlockFuel>>,
        fuel_selector: impl FuelCostsSelector,
    ) -> Result<Pos<T>, Error> {
        self.try_encode_staged()?;
        self.bump_fuel_consumption(fuel_pos, fuel_selector)?;
        let pos = self.encode_impl(op)?;
        debug_assert!(self.ops.take_reporting_pos().is_none());
        debug_assert!(self.staged.is_none());
        Ok(pos)
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
        let Some(ReportingPos::BlockFuel(pos)) = self.ops.take_reporting_pos() else {
            unreachable!("expected encoded `BlockFuel` entry but found none")
        };
        debug_assert!(self.staged.is_none());
        Ok(Some(pos))
    }

    /// Encodes a type with [`BranchOffset`] to the [`OpEncoder`] and returns its [`Pos<Op>`] and [`Pos<BranchOffset>`].
    ///
    /// # Note
    ///
    /// Bumps the `fuel` of the [`Op::ConsumeFuel`] accordingly.
    pub fn encode_branch<T>(
        &mut self,
        dst: LabelRef,
        make_branch: impl FnOnce(BranchOffset) -> T,
        fuel_pos: Option<Pos<BlockFuel>>,
        fuel_selector: impl FuelCostsSelector,
    ) -> Result<(Pos<T>, Pos<BranchOffset>), Error>
    where
        T: ir::Encode + UpdateBranchOffset,
    {
        self.try_encode_staged()?;
        self.bump_fuel_consumption(fuel_pos, fuel_selector)?;
        let offset = self.try_resolve_label(dst)?;
        let item = make_branch(offset);
        let pos_item = self.encode_impl(item)?;
        let pos_offset = match self.ops.take_reporting_pos() {
            Some(ReportingPos::BranchOffset(pos)) => pos,
            _ => panic!("missing encoded position for `BranchOffset`"),
        };
        if !self.labels.is_pinned(dst) {
            self.labels
                .new_user(dst, BytePos::from(pos_item), pos_offset);
        }
        debug_assert!(self.staged.is_none());
        Ok((pos_item, pos_offset))
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
        let fuel_used = self
            .fuel_costs
            .as_ref()
            .map(|costs| fuel_selector.select(costs))
            .unwrap_or(0);
        if fuel_used == 0 {
            return Ok(());
        }
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
        delta: FuelUsed,
    ) -> Result<(), Error> {
        debug_assert_eq!(fuel_pos.is_some(), self.fuel_costs.is_some());
        let fuel_pos = match fuel_pos {
            None => return Ok(()),
            Some(fuel_pos) => fuel_pos,
        };
        self.ops
            .update_encoded(fuel_pos, |mut fuel| -> Option<BlockFuel> {
                fuel.bump_by(delta).ok()?;
                Some(fuel)
            });
        Ok(())
    }

    /// Returns an iterator yielding all encoded [`Op`]s of the [`OpEncoder`] as bytes.
    pub fn encoded_ops(&mut self) -> &[u8] {
        debug_assert!(self.staged.is_none());
        &self.ops.buffer[..]
    }

    /// Updates the branch offsets of all branch instructions inplace.
    ///
    /// # Panics
    ///
    /// If this is used before all branching labels have been pinned.
    pub fn update_branch_offsets(&mut self) -> Result<(), Error> {
        for user in self.labels.resolved_users() {
            let ResolvedLabelUser { src, dst, pos } = user;
            let offset = trace_branch_offset(src, dst)?;
            self.ops.update_branch_offset(pos, offset)?;
        }
        Ok(())
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
        *self
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

impl EncodedOps {
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
        let Some(buffer) = self.buffer.get_mut(at..) else {
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
    fn select(self, costs: &FuelCostsProvider) -> FuelUsed;
}

impl<T> FuelCostsSelector for T
where
    T: FnOnce(&FuelCostsProvider) -> FuelUsed,
{
    fn select(self, costs: &FuelCostsProvider) -> FuelUsed {
        self(costs)
    }
}

impl FuelCostsSelector for BlockFuel {
    fn select(self, _costs: &FuelCostsProvider) -> FuelUsed {
        FuelUsed::from(self)
    }
}

impl FuelCostsSelector for FuelUsed {
    fn select(self, _costs: &FuelCostsProvider) -> FuelUsed {
        self
    }
}

/// Encodes an [`ir::OpCode`] to a generic [`ir::Encoder`].
fn encode_op_code<E: ir::Encoder>(encoder: &mut E, code: ir::OpCode) -> Result<E::Pos, E::Error> {
    match cfg!(feature = "compact") {
        true => {
            // Note: encoding for indirect-threading
            //
            // The op-codes are not resolved during translation time and must
            // be resolved during execution time. This decreases memory footprint
            // of the encoded IR at the cost of execution performance.
            u16::from(code).encode(encoder)
        }
        false => {
            // Note: encoding for direct-threading
            //
            // The op-codes are resolved during translation time (now) to their
            // underlying function pointers. This increases memory footprint
            // of the encoded IR but improves execution performance.
            (op_code_to_handler(code) as usize).encode(encoder)
        }
    }
}

/// Creates an initialized [`BranchOffset`] from `src` to `dst`.
///
/// # Errors
///
/// If the resulting [`BranchOffset`] is out of bounds.
fn trace_branch_offset(src: BytePos, dst: Pos<Op>) -> Result<BranchOffset, Error> {
    fn trace_offset_or_none(src: BytePos, dst: BytePos) -> Option<BranchOffset> {
        let src = isize::try_from(usize::from(src)).ok()?;
        let dst = isize::try_from(usize::from(dst)).ok()?;
        let offset = dst.checked_sub(src)?;
        i32::try_from(offset).map(BranchOffset::from).ok()
    }
    let Some(offset) = trace_offset_or_none(src, BytePos::from(dst)) else {
        return Err(Error::from(TranslationError::BranchOffsetOutOfBounds));
    };
    Ok(offset)
}
