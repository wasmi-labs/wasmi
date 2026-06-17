use super::{Operand, Reset};
use crate::{
    ValType,
    engine::{
        BlockType,
        translator::func::{Pos, labels::LabelRef, stack::operands::RegisterMap},
    },
    ir,
    ir::BoundedSlotSpan,
};
use alloc::vec::{Drain, Vec};
use core::slice;

#[cfg(doc)]
use crate::ir::Op;

/// The height of the operand stack upon entering a [`ControlFrame`].
#[derive(Debug, Copy, Clone)]
pub struct StackHeight(u16);

impl From<StackHeight> for usize {
    fn from(height: StackHeight) -> Self {
        usize::from(height.0)
    }
}

impl From<usize> for StackHeight {
    fn from(height: usize) -> Self {
        let Ok(height) = u16::try_from(height) else {
            panic!("out of bounds stack height: {height}")
        };
        Self(height)
    }
}

/// The branch parameters of a control flow `block`, `if` or `loop`.
#[derive(Debug, Copy, Clone)]
pub struct BranchParams {
    /// The span of slots for the branch params expected in temporary operands.
    temp_slots: BoundedSlotSpan,
    /// The number of branch parameter operands expected in temporary stack slots.
    temp_len: u16,
    /// The branch params expected in accumulator registers if any.
    ///
    /// # Dev. Note
    ///
    /// This can be `None` if all branch params are of type [`ValType::V128`]
    /// or if the number of branch params is zero (0).
    regs: Option<BranchParamRegs>,
}

impl BranchParams {
    /// Creates a new [`BranchParams`] from its raw parts.
    pub fn new(temp_slots: BoundedSlotSpan, temp_len: u16, regs: Option<BranchParamRegs>) -> Self {
        Self {
            temp_slots,
            temp_len,
            regs,
        }
    }

    /// Returns `true` if no branch parameters exist.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the total number of branch parameter operands.
    ///
    /// # Note
    ///
    /// This includes the branch parameter operands expected in temporary stack slots
    /// as well as the branch parameter operands expected in accumulator registers.
    pub fn len(&self) -> u16 {
        self.len_temps() + self.len_regs()
    }

    /// Returns the number of branch parameter operands expected in accumulator registers.
    pub fn len_regs(&self) -> u16 {
        self.regs.as_ref().map(BranchParamRegs::len).unwrap_or(0)
    }

    /// Returns the number of branch parameter operands expected in temporary stack slots.
    pub fn len_temps(&self) -> u16 {
        self.temp_len
    }

    /// Returns the [`RegKind`] of all branch parameter operands expected in accumulator registers.
    ///
    /// The order is reversed from the back, thus the first [`RegKind`] refers to the last operand etc.
    pub fn regs(&self) -> &[RegKind] {
        self.regs
            .as_ref()
            .map(BranchParamRegs::as_slice)
            .unwrap_or(&[])
    }

    /// Returns the [`BoundedSlotSpan`] for branch parameter operands that are expected in temporary stack slots.
    pub fn temp_slots(&self) -> BoundedSlotSpan {
        self.temp_slots
    }

    /// Returns `true` if `kind` is claimed by `self`.
    pub fn claims_reg(&self, kind: RegKind) -> bool {
        self.regs().contains(&kind)
    }
}

/// The branch parameters that are expected in accumulator registers.
#[derive(Debug, Copy, Clone)]
pub enum BranchParamRegs {
    /// The last branch parameter is expected in its accumulator.
    One(RegKind),
    /// The last 2 branch parameters are expected in their accumulators.
    ///
    /// # Note
    ///
    /// - Item at index `0` refers to the last operand, item at index `1` to the 2nd last.
    /// - All [`RegKind`] must be unequal.
    Two([RegKind; 2]),
    /// The last 3 branch parameters are expected in their accumulators.
    ///
    /// # Note
    ///
    /// - Items at indices refer to the following operands:
    ///     - index `0`: last operand
    ///     - index `1`: 2nd last operand
    ///     - index `2`: 3rd last operand
    /// - All [`RegKind`] must be unequal.
    Three([RegKind; 3]),
}

impl BranchParamRegs {
    /// Create a new [`BranchParamRegs::One`] from `ty`.
    pub fn new_one(kind: RegKind) -> Self {
        Self::One(kind)
    }

    /// Create a new [`BranchParamRegs::Two`] from `tys`.
    ///
    /// # Panics (Debug)
    ///
    /// If `tys` contains the same [`ValType`] more than once.
    pub fn new_two(kinds: [RegKind; 2]) -> Self {
        debug_assert_ne!(kinds[0], kinds[1]);
        Self::Two(kinds)
    }

    /// Create a new [`BranchParamRegs::Three`] from `tys`.
    ///
    /// # Panics (Debug)
    ///
    /// If `tys` contains the same [`ValType`] more than once.
    pub fn new_three(kinds: [RegKind; 3]) -> Self {
        debug_assert_ne!(kinds[0], kinds[1]);
        debug_assert_ne!(kinds[0], kinds[2]);
        debug_assert_ne!(kinds[1], kinds[2]);
        Self::Three(kinds)
    }

    /// Returns the number of branch params expected in accumulator registers.
    pub fn len(&self) -> u16 {
        match self {
            Self::One(_) => 1,
            Self::Two(_) => 2,
            Self::Three(_) => 3,
        }
    }

    /// Returns a slice over the type of branch params expected in accumulator registers.
    pub fn as_slice(&self) -> &[RegKind] {
        match self {
            Self::One(kind) => slice::from_ref(kind),
            Self::Two(kinds) => {
                debug_assert_ne!(kinds[0], kinds[1]);
                &kinds[..]
            }
            Self::Three(kinds) => {
                debug_assert_ne!(kinds[0], kinds[1]);
                debug_assert_ne!(kinds[0], kinds[2]);
                debug_assert_ne!(kinds[1], kinds[2]);
                &kinds[..]
            }
        }
    }
}

/// The kind of a register.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RegKind {
    /// The general purpose register for `i32` and `i64` values.
    Ireg,
    /// The `f32` register.
    Freg32,
    /// The `f64` register.
    Freg64,
}

impl RegKind {
    /// Creates a [`RegKind`] from a [`ValType`] if possible.
    ///
    /// Returns `None` if there is no register kind available to `ty`.
    pub fn new(ty: ValType) -> Option<Self> {
        let kind = match ty {
            ValType::I32 | ValType::FuncRef | ValType::ExternRef | ValType::I64 => Self::Ireg,
            ValType::F32 => Self::Freg32,
            ValType::F64 => Self::Freg64,
            ValType::V128 => return None,
        };
        Some(kind)
    }

    /// Returns `true` if `self` matches `ty`.
    pub fn matches_ty(&self, ty: ValType) -> bool {
        match self {
            RegKind::Ireg => matches!(
                ty,
                ValType::I32 | ValType::FuncRef | ValType::ExternRef | ValType::I64
            ),
            RegKind::Freg32 => matches!(ty, ValType::F32),
            RegKind::Freg64 => matches!(ty, ValType::F64),
        }
    }
}

/// The Wasm control stack.
#[derive(Debug, Default)]
pub struct ControlStack {
    /// The stack of control frames.
    frames: Vec<ControlFrame>,
    /// The current top-most [`Op::ConsumeFuel`] on the stack.
    ///
    /// # Note
    ///
    /// This is meant as cache and optimization to quickly query the top-most
    /// fuel consumption instruction since this information is accessed commonly.
    ///
    /// [`Op`]: crate::ir::Op
    fuel_pos: Option<Pos<ir::BlockFuel>>,
    /// Special operand stack to memorize operands for `else` control frames.
    else_operands: ElseOperands,
    /// This is `true` if an `if` with else providers was just popped from the stack.
    ///
    /// # Note
    ///
    /// This means that its associated `else` operands need to be taken care of by
    /// either pushing back an `else` control frame or by manually popping them off
    /// the control stack.
    orphaned_else_operands: bool,
}

/// Duplicated operands for Wasm `else` control frames.
#[derive(Debug, Default)]
pub struct ElseOperands {
    /// The end indices of each `else` operands.
    ends: Vec<usize>,
    /// All operands of all allocated `else` control frames.
    operands: Vec<Operand>,
}

impl Reset for ElseOperands {
    fn reset(&mut self) {
        self.ends.clear();
        self.operands.clear();
    }
}

impl ElseOperands {
    /// Pushes operands for a new Wasm `else` control frame.
    pub fn push(&mut self, operands: impl IntoIterator<Item = Operand>) {
        self.operands.extend(operands);
        let end = self.operands.len();
        self.ends.push(end);
    }

    /// Pops the top-most Wasm `else` operands from `self` and returns them.
    pub fn pop(&mut self) -> Option<Drain<'_, Operand>> {
        let end = self.ends.pop()?;
        let start = self.ends.last().copied().unwrap_or(0);
        Some(self.operands.drain(start..end))
    }
}

impl Reset for ControlStack {
    fn reset(&mut self) {
        self.frames.clear();
        self.else_operands.reset();
        self.orphaned_else_operands = false;
    }
}

impl ControlStack {
    /// Returns the current [`Op::ConsumeFuel`] if fuel metering is enabled.
    ///
    /// Returns `None` otherwise.
    #[inline]
    pub fn fuel_pos(&self) -> Option<Pos<ir::BlockFuel>> {
        debug_assert!(!self.is_empty());
        self.fuel_pos
    }

    /// Returns `true` if `self` is empty.
    pub fn is_empty(&self) -> bool {
        self.height() == 0
    }

    /// Returns the height of the [`ControlStack`].
    pub fn height(&self) -> usize {
        self.frames.len()
    }

    /// Pushes a new unreachable Wasm control frame onto the [`ControlStack`].
    pub fn push_unreachable(&mut self, kind: ControlFrameKind) {
        debug_assert!(!self.orphaned_else_operands);
        self.frames.push(ControlFrame::from(kind))
    }

    /// Pushes a new Wasm `block` onto the [`ControlStack`].
    pub fn push_block(
        &mut self,
        ty: BlockType,
        height: usize,
        branch_params: BranchParams,
        label: LabelRef,
        fuel_pos: Option<Pos<ir::BlockFuel>>,
    ) {
        debug_assert!(!self.orphaned_else_operands);
        self.frames.push(ControlFrame::from(BlockControlFrame {
            ty,
            height: StackHeight::from(height),
            branch_params,
            is_branched_to: false,
            fuel_pos,
            label,
        }));
        self.fuel_pos = fuel_pos;
    }

    /// Pushes a new Wasm `loop` onto the [`ControlStack`].
    pub fn push_loop(
        &mut self,
        ty: BlockType,
        height: usize,
        branch_params: BranchParams,
        label: LabelRef,
        fuel_pos: Option<Pos<ir::BlockFuel>>,
    ) -> LoopControlFrame {
        debug_assert!(!self.orphaned_else_operands);
        let loop_frame = LoopControlFrame {
            ty,
            height: StackHeight::from(height),
            branch_params,
            is_branched_to: false,
            fuel_pos,
            label,
        };
        self.frames.push(ControlFrame::from(loop_frame.clone()));
        self.fuel_pos = fuel_pos;
        loop_frame
    }

    /// Pushes a new Wasm `if` onto the [`ControlStack`].
    #[expect(clippy::too_many_arguments)]
    pub fn push_if(
        &mut self,
        ty: BlockType,
        height: usize,
        branch_params: BranchParams,
        label: LabelRef,
        fuel_pos: Option<Pos<ir::BlockFuel>>,
        reachability: IfReachability,
        else_operands: impl IntoIterator<Item = Operand>,
        registers: RegisterMap,
    ) {
        debug_assert!(!self.orphaned_else_operands);
        self.frames.push(ControlFrame::from(IfControlFrame {
            ty,
            height: StackHeight::from(height),
            branch_params,
            is_branched_to: false,
            fuel_pos,
            label,
            reachability,
            registers,
        }));
        if matches!(reachability, IfReachability::Both { .. }) {
            self.else_operands.push(else_operands);
        }
        self.fuel_pos = fuel_pos;
    }

    /// Pushes a new Wasm `else` onto the [`ControlStack`].
    ///
    /// Returns iterator yielding the memorized `else` operands.
    pub fn push_else(
        &mut self,
        if_frame: IfControlFrame,
        fuel_pos: Option<Pos<ir::BlockFuel>>,
        is_end_of_then_reachable: bool,
    ) {
        debug_assert!(!self.orphaned_else_operands);
        let ty = if_frame.ty();
        let height = if_frame.height();
        let branch_params = if_frame.branch_params;
        let label = if_frame.label();
        let is_branched_to = if_frame.is_branched_to();
        let reachability = match if_frame.reachability {
            IfReachability::Both { .. } => ElseReachability::Both {
                is_end_of_then_reachable,
            },
            IfReachability::OnlyThen => ElseReachability::OnlyThen {
                is_end_of_then_reachable,
            },
            IfReachability::OnlyElse => ElseReachability::OnlyElse,
        };
        self.frames.push(ControlFrame::from(ElseControlFrame {
            ty,
            height: StackHeight::from(height),
            branch_params,
            is_branched_to,
            fuel_pos,
            label,
            reachability,
        }));
        self.fuel_pos = fuel_pos;
    }

    /// Pops the top-most [`ControlFrame`] and returns it if any.
    pub fn pop(&mut self) -> Option<ControlFrame> {
        debug_assert!(!self.orphaned_else_operands);
        let frame = self.frames.pop()?;
        if !matches!(frame, ControlFrame::Block(_) | ControlFrame::Unreachable(_)) {
            // Need to replace the cached top-most `fuel_pos`.
            self.fuel_pos = self.get(0).fuel_pos();
        }
        self.orphaned_else_operands = match &frame {
            ControlFrame::If(frame) => {
                matches!(frame.reachability, IfReachability::Both { .. })
            }
            _ => false,
        };
        Some(frame)
    }

    /// Pops the top-most `else` operands from the control stack.
    ///
    /// # Panics (Debug)
    ///
    /// If the `else` operands are not in orphaned state.
    pub fn pop_else_operands(&mut self) -> Drain<'_, Operand> {
        debug_assert!(self.orphaned_else_operands);
        let Some(else_operands) = self.else_operands.pop() else {
            panic!("missing `else` operands")
        };
        self.orphaned_else_operands = false;
        else_operands
    }

    /// Returns a shared reference to the [`ControlFrame`] at `depth` if any.
    pub fn get(&self, depth: usize) -> &ControlFrame {
        let height = self.height();
        self.frames.iter().rev().nth(depth).unwrap_or_else(|| {
            panic!(
                "out of bounds control frame at depth (={depth}) for stack of height (={height})"
            )
        })
    }

    /// Returns an exclusive reference to the [`ControlFrame`] at `depth` if any.
    pub fn get_mut(&mut self, depth: usize) -> ControlFrameMut<'_> {
        let height = self.height();
        self.frames
            .iter_mut()
            .rev()
            .nth(depth)
            .map(ControlFrameMut)
            .unwrap_or_else(|| {
                panic!(
                "out of bounds control frame at depth (={depth}) for stack of height (={height})"
            )
            })
    }
}

/// An exclusive reference to a [`ControlFrame`].
#[derive(Debug)]
pub struct ControlFrameMut<'a>(&'a mut ControlFrame);

impl<'a> ControlFrameBase for ControlFrameMut<'a> {
    fn kind(&self) -> ControlFrameKind {
        self.0.kind()
    }

    fn ty(&self) -> BlockType {
        self.0.ty()
    }

    fn height(&self) -> usize {
        self.0.height()
    }

    fn branch_params(&self) -> BranchParams {
        self.0.branch_params()
    }

    fn label(&self) -> LabelRef {
        self.0.label()
    }

    fn is_branched_to(&self) -> bool {
        self.0.is_branched_to()
    }

    fn branch_to(&mut self) {
        self.0.branch_to()
    }

    fn fuel_pos(&self) -> Option<Pos<ir::BlockFuel>> {
        self.0.fuel_pos()
    }
}

/// An acquired branch target.
#[derive(Debug)]
pub enum AcquiredTarget<'stack> {
    /// The branch targets the function enclosing `block` and therefore is a `return`.
    Return(ControlFrameMut<'stack>),
    /// The branch targets a regular [`ControlFrame`].
    Branch(ControlFrameMut<'stack>),
}

impl<'stack> AcquiredTarget<'stack> {
    /// Returns an exclusive reference to the [`ControlFrame`] of the [`AcquiredTarget`].
    pub fn control_frame(self) -> ControlFrameMut<'stack> {
        match self {
            Self::Return(frame) => frame,
            Self::Branch(frame) => frame,
        }
    }
}

impl ControlStack {
    /// Acquires the target [`ControlFrame`] at the given relative `depth`.
    pub fn acquire_target(&mut self, depth: usize) -> AcquiredTarget<'_> {
        let is_root = self.is_root(depth);
        let frame = self.get_mut(depth);
        if is_root {
            AcquiredTarget::Return(frame)
        } else {
            AcquiredTarget::Branch(frame)
        }
    }

    /// Returns `true` if `depth` points to the first control flow frame.
    fn is_root(&self, depth: usize) -> bool {
        if self.frames.is_empty() {
            return false;
        }
        depth == self.height() - 1
    }
}

/// A Wasm control frame.
#[derive(Debug)]
pub enum ControlFrame {
    /// A Wasm `block` control frame.
    Block(BlockControlFrame),
    /// A Wasm `loop` control frame.
    Loop(LoopControlFrame),
    /// A Wasm `if` control frame.
    If(IfControlFrame),
    /// A Wasm `else` control frame.
    Else(ElseControlFrame),
    /// A generic unreachable control frame.
    Unreachable(ControlFrameKind),
}

impl From<BlockControlFrame> for ControlFrame {
    fn from(frame: BlockControlFrame) -> Self {
        Self::Block(frame)
    }
}

impl From<LoopControlFrame> for ControlFrame {
    fn from(frame: LoopControlFrame) -> Self {
        Self::Loop(frame)
    }
}

impl From<IfControlFrame> for ControlFrame {
    fn from(frame: IfControlFrame) -> Self {
        Self::If(frame)
    }
}

impl From<ElseControlFrame> for ControlFrame {
    fn from(frame: ElseControlFrame) -> Self {
        Self::Else(frame)
    }
}

impl From<ControlFrameKind> for ControlFrame {
    fn from(frame: ControlFrameKind) -> Self {
        Self::Unreachable(frame)
    }
}

/// Trait implemented by control frame types that share a common API.
pub trait ControlFrameBase {
    /// Returns the [`ControlFrameKind`] of the control frame.
    fn kind(&self) -> ControlFrameKind;

    /// Returns the [`BlockType`] of the control frame.
    fn ty(&self) -> BlockType;

    /// Returns the height of the control frame.
    fn height(&self) -> usize;

    /// Returns the [`BranchParams`] of the control frame.
    fn branch_params(&self) -> BranchParams;

    /// Returns the branch label of `self`.
    fn label(&self) -> LabelRef;

    /// Returns `true` if there exists a branch to `self.`
    fn is_branched_to(&self) -> bool;

    /// Makes `self` aware that there is a branch to it.
    fn branch_to(&mut self);

    /// Returns a reference to the [`Op::ConsumeFuel`] of `self`.
    ///
    /// Returns `None` if fuel metering is disabled.
    fn fuel_pos(&self) -> Option<Pos<ir::BlockFuel>>;
}

impl ControlFrameBase for ControlFrame {
    fn kind(&self) -> ControlFrameKind {
        match self {
            ControlFrame::Block(frame) => frame.kind(),
            ControlFrame::Loop(frame) => frame.kind(),
            ControlFrame::If(frame) => frame.kind(),
            ControlFrame::Else(frame) => frame.kind(),
            ControlFrame::Unreachable(_) => {
                panic!("invalid query for unreachable control frame: `ControlFrameBase::kind`")
            }
        }
    }

    fn ty(&self) -> BlockType {
        match self {
            ControlFrame::Block(frame) => frame.ty(),
            ControlFrame::Loop(frame) => frame.ty(),
            ControlFrame::If(frame) => frame.ty(),
            ControlFrame::Else(frame) => frame.ty(),
            ControlFrame::Unreachable(_) => {
                panic!("invalid query for unreachable control frame: `ControlFrameBase::ty`")
            }
        }
    }

    fn height(&self) -> usize {
        match self {
            ControlFrame::Block(frame) => frame.height(),
            ControlFrame::Loop(frame) => frame.height(),
            ControlFrame::If(frame) => frame.height(),
            ControlFrame::Else(frame) => frame.height(),
            ControlFrame::Unreachable(_) => {
                panic!("invalid query for unreachable control frame: `ControlFrameBase::height`")
            }
        }
    }

    fn branch_params(&self) -> BranchParams {
        match self {
            ControlFrame::Block(frame) => frame.branch_params(),
            ControlFrame::Loop(frame) => frame.branch_params(),
            ControlFrame::If(frame) => frame.branch_params(),
            ControlFrame::Else(frame) => frame.branch_params(),
            ControlFrame::Unreachable(_) => {
                panic!(
                    "invalid query for unreachable control frame: `ControlFrameBase::branch_params`"
                )
            }
        }
    }

    fn label(&self) -> LabelRef {
        match self {
            ControlFrame::Block(frame) => frame.label(),
            ControlFrame::Loop(frame) => frame.label(),
            ControlFrame::If(frame) => frame.label(),
            ControlFrame::Else(frame) => frame.label(),
            ControlFrame::Unreachable(_) => {
                panic!("invalid query for unreachable control frame: `ControlFrame::label`")
            }
        }
    }

    fn is_branched_to(&self) -> bool {
        match self {
            ControlFrame::Block(frame) => frame.is_branched_to(),
            ControlFrame::Loop(frame) => frame.is_branched_to(),
            ControlFrame::If(frame) => frame.is_branched_to(),
            ControlFrame::Else(frame) => frame.is_branched_to(),
            ControlFrame::Unreachable(_) => {
                panic!(
                    "invalid query for unreachable control frame: `ControlFrame::is_branched_to`"
                )
            }
        }
    }

    fn branch_to(&mut self) {
        match self {
            ControlFrame::Block(frame) => frame.branch_to(),
            ControlFrame::Loop(frame) => frame.branch_to(),
            ControlFrame::If(frame) => frame.branch_to(),
            ControlFrame::Else(frame) => frame.branch_to(),
            ControlFrame::Unreachable(_) => {
                panic!("invalid query for unreachable control frame: `ControlFrame::branch_to`")
            }
        }
    }

    fn fuel_pos(&self) -> Option<Pos<ir::BlockFuel>> {
        match self {
            ControlFrame::Block(frame) => frame.fuel_pos(),
            ControlFrame::Loop(frame) => frame.fuel_pos(),
            ControlFrame::If(frame) => frame.fuel_pos(),
            ControlFrame::Else(frame) => frame.fuel_pos(),
            ControlFrame::Unreachable(_) => {
                panic!("invalid query for unreachable control frame: `ControlFrame::fuel_pos`")
            }
        }
    }
}

/// A Wasm `block` control frame.
#[derive(Debug)]
pub struct BlockControlFrame {
    /// The block type of the [`BlockControlFrame`].
    ty: BlockType,
    /// The value stack height upon entering the [`BlockControlFrame`].
    height: StackHeight,
    /// The [`BranchParams`] of the [`BlockControlFrame`].
    branch_params: BranchParams,
    /// This is `true` if there is at least one branch to this [`BlockControlFrame`].
    is_branched_to: bool,
    /// The [`BlockControlFrame`]'s [`Op::ConsumeFuel`] if fuel metering is enabled.
    ///
    /// # Note
    ///
    /// This is `Some` if fuel metering is enabled and `None` otherwise.
    fuel_pos: Option<Pos<ir::BlockFuel>>,
    /// The label used to branch to the [`BlockControlFrame`].
    label: LabelRef,
}

impl ControlFrameBase for BlockControlFrame {
    fn kind(&self) -> ControlFrameKind {
        ControlFrameKind::Block
    }

    fn ty(&self) -> BlockType {
        self.ty
    }

    fn height(&self) -> usize {
        self.height.into()
    }

    fn branch_params(&self) -> BranchParams {
        self.branch_params
    }

    fn label(&self) -> LabelRef {
        self.label
    }

    fn is_branched_to(&self) -> bool {
        self.is_branched_to
    }

    fn branch_to(&mut self) {
        self.is_branched_to = true;
    }

    fn fuel_pos(&self) -> Option<Pos<ir::BlockFuel>> {
        self.fuel_pos
    }
}

/// A Wasm `loop` control frame.
#[derive(Debug, Clone)]
pub struct LoopControlFrame {
    /// The block type of the [`LoopControlFrame`].
    ty: BlockType,
    /// The value stack height upon entering the [`LoopControlFrame`].
    height: StackHeight,
    /// The [`BranchParams`] of the [`LoopControlFrame`].
    branch_params: BranchParams,
    /// This is `true` if there is at least one branch to this [`LoopControlFrame`].
    is_branched_to: bool,
    /// The [`LoopControlFrame`]'s [`Op::ConsumeFuel`] if fuel metering is enabled.
    ///
    /// # Note
    ///
    /// This is `Some` if fuel metering is enabled and `None` otherwise.
    fuel_pos: Option<Pos<ir::BlockFuel>>,
    /// The label used to branch to the [`LoopControlFrame`].
    label: LabelRef,
}

impl ControlFrameBase for LoopControlFrame {
    fn kind(&self) -> ControlFrameKind {
        ControlFrameKind::Loop
    }

    fn ty(&self) -> BlockType {
        self.ty
    }

    fn height(&self) -> usize {
        self.height.into()
    }

    fn branch_params(&self) -> BranchParams {
        self.branch_params
    }

    fn label(&self) -> LabelRef {
        self.label
    }

    fn is_branched_to(&self) -> bool {
        self.is_branched_to
    }

    fn branch_to(&mut self) {
        self.is_branched_to = true;
    }

    fn fuel_pos(&self) -> Option<Pos<ir::BlockFuel>> {
        self.fuel_pos
    }
}

/// A Wasm `if` control frame including its `then` part.
#[derive(Debug)]
pub struct IfControlFrame {
    /// The block type of the [`IfControlFrame`].
    ty: BlockType,
    /// The value stack height upon entering the [`IfControlFrame`].
    height: StackHeight,
    /// The [`BranchParams`] of the [`IfControlFrame`].
    branch_params: BranchParams,
    /// This is `true` if there is at least one branch to this [`IfControlFrame`].
    is_branched_to: bool,
    /// The [`IfControlFrame`]'s [`Op::ConsumeFuel`] if fuel metering is enabled.
    ///
    /// # Note
    ///
    /// This is `Some` if fuel metering is enabled and `None` otherwise.
    fuel_pos: Option<Pos<ir::BlockFuel>>,
    /// The label used to branch to the [`IfControlFrame`].
    label: LabelRef,
    /// The reachability of the `then` and `else` blocks.
    reachability: IfReachability,
    /// The state of registers on the stack upon entering the `if` block.
    ///
    /// This is used to restore the register state upon entering the `else` block.
    registers: RegisterMap,
}

impl IfControlFrame {
    /// Returns the [`IfReachability`] of the [`IfControlFrame`].
    pub fn reachability(&self) -> IfReachability {
        self.reachability
    }

    /// Returns `true` if the `else` branch is reachable.
    ///
    /// # Note
    ///
    /// The `else` branch is unreachable if the `if` condition is a constant `true` value.
    pub fn is_else_reachable(&self) -> bool {
        match self.reachability {
            IfReachability::Both { .. } | IfReachability::OnlyElse => true,
            IfReachability::OnlyThen => false,
        }
    }

    /// Returns the state of registers on the stack upon entering the `if` block.
    pub(super) fn registers(&self) -> RegisterMap {
        self.registers
    }
}

impl ControlFrameBase for IfControlFrame {
    fn kind(&self) -> ControlFrameKind {
        ControlFrameKind::If
    }

    fn ty(&self) -> BlockType {
        self.ty
    }

    fn height(&self) -> usize {
        self.height.into()
    }

    fn branch_params(&self) -> BranchParams {
        self.branch_params
    }

    fn label(&self) -> LabelRef {
        self.label
    }

    fn is_branched_to(&self) -> bool {
        self.is_branched_to
    }

    fn branch_to(&mut self) {
        self.is_branched_to = true;
    }

    fn fuel_pos(&self) -> Option<Pos<ir::BlockFuel>> {
        self.fuel_pos
    }
}

/// The reachability of the `if` control flow frame.
#[derive(Debug, Copy, Clone)]
pub enum IfReachability {
    /// Both, `then` and `else` blocks of the `if` are reachable.
    ///
    /// # Note
    ///
    /// This variant does not mean that necessarily both `then` and `else`
    /// blocks do exist and are non-empty. The `then` block might still be
    /// empty and the `then` block might still be missing.
    Both { else_label: LabelRef },
    /// Only the `then` block of the `if` is reachable.
    ///
    /// # Note
    ///
    /// This case happens only in case the `if` has a `true` constant condition.
    OnlyThen,
    /// Only the `else` block of the `if` is reachable.
    ///
    /// # Note
    ///
    /// This case happens only in case the `if` has a `false` constant condition.
    OnlyElse,
}

/// A Wasm `else` control frame part of Wasm `if`.
#[derive(Debug)]
pub struct ElseControlFrame {
    /// The block type of the [`ElseControlFrame`].
    ty: BlockType,
    /// The value stack height upon entering the [`ElseControlFrame`].
    height: StackHeight,
    /// The [`BranchParams`] of the [`ElseControlFrame`].
    branch_params: BranchParams,
    /// This is `true` if there is at least one branch to this [`ElseControlFrame`].
    is_branched_to: bool,
    /// The [`LoopControlFrame`]'s [`Op::ConsumeFuel`] if fuel metering is enabled.
    ///
    /// # Note
    ///
    /// This is `Some` if fuel metering is enabled and `None` otherwise.
    fuel_pos: Option<Pos<ir::BlockFuel>>,
    /// The label used to branch to the [`ElseControlFrame`].
    label: LabelRef,
    /// The reachability of the `then` and `else` blocks.
    reachability: ElseReachability,
}

/// The reachability of the `else` control flow frame.
#[derive(Debug, Copy, Clone)]
pub enum ElseReachability {
    /// Both, `then` and `else` blocks of the `if` are reachable.
    ///
    /// # Note
    ///
    /// This variant does not mean that necessarily both `then` and `else`
    /// blocks do exist and are non-empty. The `then` block might still be
    /// empty and the `then` block might still be missing.
    Both {
        /// Is `true` if code is reachable when entering the `else` block.
        ///
        /// # Note
        ///
        /// This means that the end of the `then` block was reachable.
        is_end_of_then_reachable: bool,
    },
    /// Only the `then` block of the `if` is reachable.
    ///
    /// # Note
    ///
    /// This case happens only in case the `if` has a `true` constant condition.
    OnlyThen {
        /// Is `true` if code is reachable when entering the `else` block.
        ///
        /// # Note
        ///
        /// This means that the end of the `then` block was reachable.
        is_end_of_then_reachable: bool,
    },
    /// Only the `else` block of the `if` is reachable.
    ///
    /// # Note
    ///
    /// This case happens only in case the `if` has a `false` constant condition.
    OnlyElse,
}

impl ElseControlFrame {
    /// Returns the [`ElseReachability`] of the [`ElseReachability`].
    pub fn reachability(&self) -> ElseReachability {
        self.reachability
    }

    /// Returns `true` if the end of the `then` branch is reachable.
    pub fn is_end_of_then_reachable(&self) -> bool {
        match self.reachability {
            ElseReachability::Both {
                is_end_of_then_reachable,
            }
            | ElseReachability::OnlyThen {
                is_end_of_then_reachable,
            } => is_end_of_then_reachable,
            ElseReachability::OnlyElse => false,
        }
    }
}

impl ControlFrameBase for ElseControlFrame {
    fn kind(&self) -> ControlFrameKind {
        ControlFrameKind::Else
    }

    fn ty(&self) -> BlockType {
        self.ty
    }

    fn height(&self) -> usize {
        self.height.into()
    }

    fn branch_params(&self) -> BranchParams {
        self.branch_params
    }

    fn label(&self) -> LabelRef {
        self.label
    }

    fn is_branched_to(&self) -> bool {
        self.is_branched_to
    }

    fn branch_to(&mut self) {
        self.is_branched_to = true;
    }

    fn fuel_pos(&self) -> Option<Pos<ir::BlockFuel>> {
        self.fuel_pos
    }
}

/// The kind of a Wasm control frame.
#[derive(Debug, Copy, Clone)]
pub enum ControlFrameKind {
    /// An Wasm `block` control frame.
    Block,
    /// An Wasm `loop` control frame.
    Loop,
    /// An Wasm `if` control frame.
    If,
    /// An Wasm `else` control frame.
    Else,
}
