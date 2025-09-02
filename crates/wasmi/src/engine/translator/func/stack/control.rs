use super::{Operand, Reset};
use crate::{
    engine::{
        translator::{labels::LabelRef, utils::Instr},
        BlockType,
    },
    Engine,
};
use alloc::vec::{Drain, Vec};

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
    consume_fuel_instr: Option<Instr>,
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
    pub fn consume_fuel_instr(&self) -> Option<Instr> {
        debug_assert!(!self.is_empty());
        self.consume_fuel_instr
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
        label: LabelRef,
        consume_fuel: Option<Instr>,
    ) {
        debug_assert!(!self.orphaned_else_operands);
        self.frames.push(ControlFrame::from(BlockControlFrame {
            ty,
            height: StackHeight::from(height),
            is_branched_to: false,
            consume_fuel,
            label,
        }));
        self.consume_fuel_instr = consume_fuel;
    }

    /// Pushes a new Wasm `loop` onto the [`ControlStack`].
    pub fn push_loop(
        &mut self,
        ty: BlockType,
        height: usize,
        label: LabelRef,
        consume_fuel: Option<Instr>,
    ) {
        debug_assert!(!self.orphaned_else_operands);
        self.frames.push(ControlFrame::from(LoopControlFrame {
            ty,
            height: StackHeight::from(height),
            is_branched_to: false,
            consume_fuel,
            label,
        }));
        self.consume_fuel_instr = consume_fuel;
    }

    /// Pushes a new Wasm `if` onto the [`ControlStack`].
    pub fn push_if(
        &mut self,
        ty: BlockType,
        height: usize,
        label: LabelRef,
        consume_fuel: Option<Instr>,
        reachability: IfReachability,
        else_operands: impl IntoIterator<Item = Operand>,
    ) {
        debug_assert!(!self.orphaned_else_operands);
        self.frames.push(ControlFrame::from(IfControlFrame {
            ty,
            height: StackHeight::from(height),
            is_branched_to: false,
            consume_fuel,
            label,
            reachability,
        }));
        if matches!(reachability, IfReachability::Both { .. }) {
            self.else_operands.push(else_operands);
        }
        self.consume_fuel_instr = consume_fuel;
    }

    /// Pushes a new Wasm `else` onto the [`ControlStack`].
    ///
    /// Returns iterator yielding the memorized `else` operands.
    pub fn push_else(
        &mut self,
        if_frame: IfControlFrame,
        consume_fuel: Option<Instr>,
        is_end_of_then_reachable: bool,
    ) {
        debug_assert!(!self.orphaned_else_operands);
        let ty = if_frame.ty();
        let height = if_frame.height();
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
            is_branched_to,
            consume_fuel,
            label,
            reachability,
        }));
        self.consume_fuel_instr = consume_fuel;
    }

    /// Pops the top-most [`ControlFrame`] and returns it if any.
    pub fn pop(&mut self) -> Option<ControlFrame> {
        debug_assert!(!self.orphaned_else_operands);
        let frame = self.frames.pop()?;
        if !matches!(frame, ControlFrame::Block(_) | ControlFrame::Unreachable(_)) {
            // Need to replace the cached top-most `consume_fuel_instr`.
            self.consume_fuel_instr = self.get(0).consume_fuel_instr();
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
    fn ty(&self) -> BlockType {
        self.0.ty()
    }

    fn height(&self) -> usize {
        self.0.height()
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

    fn len_branch_params(&self, engine: &Engine) -> u16 {
        self.0.len_branch_params(engine)
    }

    fn consume_fuel_instr(&self) -> Option<Instr> {
        self.0.consume_fuel_instr()
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
    /// Returns the [`BlockType`] of the [`BlockControlFrame`].
    fn ty(&self) -> BlockType;

    /// Returns the height of the [`BlockControlFrame`].
    fn height(&self) -> usize;

    /// Returns the branch label of `self`.
    fn label(&self) -> LabelRef;

    /// Returns `true` if there exists a branch to `self.`
    fn is_branched_to(&self) -> bool;

    /// Makes `self` aware that there is a branch to it.
    fn branch_to(&mut self);

    /// Returns the number of operands required for branching to `self`.
    fn len_branch_params(&self, engine: &Engine) -> u16;

    /// Returns a reference to the [`Op::ConsumeFuel`] of `self`.
    ///
    /// Returns `None` if fuel metering is disabled.
    fn consume_fuel_instr(&self) -> Option<Instr>;
}

impl ControlFrameBase for ControlFrame {
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

    fn len_branch_params(&self, engine: &Engine) -> u16 {
        match self {
            ControlFrame::Block(frame) => frame.len_branch_params(engine),
            ControlFrame::Loop(frame) => frame.len_branch_params(engine),
            ControlFrame::If(frame) => frame.len_branch_params(engine),
            ControlFrame::Else(frame) => frame.len_branch_params(engine),
            ControlFrame::Unreachable(_) => {
                panic!("invalid query for unreachable control frame: `ControlFrame::len_branch_params`")
            }
        }
    }

    fn consume_fuel_instr(&self) -> Option<Instr> {
        match self {
            ControlFrame::Block(frame) => frame.consume_fuel_instr(),
            ControlFrame::Loop(frame) => frame.consume_fuel_instr(),
            ControlFrame::If(frame) => frame.consume_fuel_instr(),
            ControlFrame::Else(frame) => frame.consume_fuel_instr(),
            ControlFrame::Unreachable(_) => {
                panic!("invalid query for unreachable control frame: `ControlFrame::consume_fuel_instr`")
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
    /// This is `true` if there is at least one branch to this [`BlockControlFrame`].
    is_branched_to: bool,
    /// The [`BlockControlFrame`]'s [`Op::ConsumeFuel`] if fuel metering is enabled.
    ///
    /// # Note
    ///
    /// This is `Some` if fuel metering is enabled and `None` otherwise.
    consume_fuel: Option<Instr>,
    /// The label used to branch to the [`BlockControlFrame`].
    label: LabelRef,
}

impl ControlFrameBase for BlockControlFrame {
    fn ty(&self) -> BlockType {
        self.ty
    }

    fn height(&self) -> usize {
        self.height.into()
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

    fn len_branch_params(&self, engine: &Engine) -> u16 {
        self.ty.len_results(engine)
    }

    fn consume_fuel_instr(&self) -> Option<Instr> {
        self.consume_fuel
    }
}

/// A Wasm `loop` control frame.
#[derive(Debug)]
pub struct LoopControlFrame {
    /// The block type of the [`LoopControlFrame`].
    ty: BlockType,
    /// The value stack height upon entering the [`LoopControlFrame`].
    height: StackHeight,
    /// This is `true` if there is at least one branch to this [`LoopControlFrame`].
    is_branched_to: bool,
    /// The [`LoopControlFrame`]'s [`Op::ConsumeFuel`] if fuel metering is enabled.
    ///
    /// # Note
    ///
    /// This is `Some` if fuel metering is enabled and `None` otherwise.
    consume_fuel: Option<Instr>,
    /// The label used to branch to the [`LoopControlFrame`].
    label: LabelRef,
}

impl ControlFrameBase for LoopControlFrame {
    fn ty(&self) -> BlockType {
        self.ty
    }

    fn height(&self) -> usize {
        self.height.into()
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

    fn len_branch_params(&self, engine: &Engine) -> u16 {
        self.ty.len_params(engine)
    }

    fn consume_fuel_instr(&self) -> Option<Instr> {
        self.consume_fuel
    }
}

/// A Wasm `if` control frame including its `then` part.
#[derive(Debug)]
pub struct IfControlFrame {
    /// The block type of the [`IfControlFrame`].
    ty: BlockType,
    /// The value stack height upon entering the [`IfControlFrame`].
    height: StackHeight,
    /// This is `true` if there is at least one branch to this [`IfControlFrame`].
    is_branched_to: bool,
    /// The [`IfControlFrame`]'s [`Op::ConsumeFuel`] if fuel metering is enabled.
    ///
    /// # Note
    ///
    /// This is `Some` if fuel metering is enabled and `None` otherwise.
    consume_fuel: Option<Instr>,
    /// The label used to branch to the [`IfControlFrame`].
    label: LabelRef,
    /// The reachability of the `then` and `else` blocks.
    reachability: IfReachability,
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
}

impl ControlFrameBase for IfControlFrame {
    fn ty(&self) -> BlockType {
        self.ty
    }

    fn height(&self) -> usize {
        self.height.into()
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

    fn len_branch_params(&self, engine: &Engine) -> u16 {
        self.ty.len_results(engine)
    }

    fn consume_fuel_instr(&self) -> Option<Instr> {
        self.consume_fuel
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
    /// This is `true` if there is at least one branch to this [`ElseControlFrame`].
    is_branched_to: bool,
    /// The [`LoopControlFrame`]'s [`Op::ConsumeFuel`] if fuel metering is enabled.
    ///
    /// # Note
    ///
    /// This is `Some` if fuel metering is enabled and `None` otherwise.
    consume_fuel: Option<Instr>,
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
    fn ty(&self) -> BlockType {
        self.ty
    }

    fn height(&self) -> usize {
        self.height.into()
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

    fn len_branch_params(&self, engine: &Engine) -> u16 {
        self.ty.len_results(engine)
    }

    fn consume_fuel_instr(&self) -> Option<Instr> {
        self.consume_fuel
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
