use super::{Operand, Reset};
use crate::engine::{
    translator::{BlockType, LabelRef},
    Instr,
};
use alloc::vec::{Drain, Vec};

#[cfg(doc)]
use crate::ir::Instruction;

/// The Wasm control stack.
#[derive(Debug, Default)]
pub struct ControlStack {
    /// The stack of control frames.
    frames: Vec<ControlFrame>,
    /// Special operand stack to memorize operands for `else` control frames.
    else_operands: ElseOperands,
}

/// Duplicated operands for Wasm `else` control frames.
#[derive(Debug, Default)]
pub struct ElseOperands {
    /// The end indices of each `else` operands.
    ends: Vec<usize>,
    /// All operands of all allocated `else` control frames.
    providers: Vec<Operand>,
}

impl Reset for ElseOperands {
    fn reset(&mut self) {
        self.ends.clear();
        self.providers.clear();
    }
}

impl ElseOperands {
    /// Pushes operands for a new Wasm `else` control frame.
    pub fn push(&mut self, operands: impl IntoIterator<Item = Operand>) {
        self.providers.extend(operands);
        let end = self.providers.len();
        let index = self.ends.len();
        self.ends.push(end);
    }

    /// Pops the top-most Wasm `else` operands from `self` and returns it.
    pub fn pop(&mut self) -> Option<Drain<Operand>> {
        let end = self.ends.pop()?;
        let start = self.ends.last().copied().unwrap_or(0);
        Some(self.providers.drain(start..end))
    }
}

impl Reset for ControlStack {
    fn reset(&mut self) {
        self.frames.clear();
        self.else_operands.reset();
    }
}

impl ControlStack {
    /// Returns the height of the [`ControlStack`].
    pub fn height(&self) -> usize {
        self.frames.len()
    }

    /// Pushes a new unreachable Wasm control frame onto the [`ControlStack`].
    pub fn push_unreachable(
        &mut self,
        ty: BlockType,
        height: usize,
        kind: UnreachableControlFrame,
    ) {
        self.frames
            .push(ControlFrame::new_unreachable(ty, height, kind))
    }

    /// Pushes a new Wasm `block` onto the [`ControlStack`].
    pub fn push_block(
        &mut self,
        ty: BlockType,
        height: usize,
        label: LabelRef,
        consume_fuel: Option<Instr>,
    ) {
        self.frames
            .push(ControlFrame::new_block(ty, height, label, consume_fuel))
    }

    /// Pushes a new Wasm `loop` onto the [`ControlStack`].
    pub fn push_loop(
        &mut self,
        ty: BlockType,
        height: usize,
        label: LabelRef,
        consume_fuel: Option<Instr>,
    ) {
        self.frames
            .push(ControlFrame::new_loop(ty, height, label, consume_fuel))
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
        self.frames.push(ControlFrame::new_if(
            ty,
            height,
            label,
            consume_fuel,
            reachability,
        ));
        self.else_operands.push(else_operands);
    }

    /// Pushes a new Wasm `else` onto the [`ControlStack`].
    ///
    /// Returns iterator yielding the memorized `else` operands.
    pub fn push_else(
        &mut self,
        ty: BlockType,
        height: usize,
        label: LabelRef,
        consume_fuel: Option<Instr>,
        reachability: ElseReachability,
        is_end_of_then_reachable: bool,
    ) -> Drain<Operand> {
        self.frames.push(ControlFrame::new_else(
            ty,
            height,
            label,
            consume_fuel,
            reachability,
            is_end_of_then_reachable,
        ));
        self.else_operands
            .pop()
            .unwrap_or_else(|| panic!("missing operands for `else` control frame"))
    }

    /// Pops the top-most [`ControlFrame`] and returns it if any.
    pub fn pop(&mut self) -> Option<ControlFrame> {
        self.frames.pop()
    }

    /// Returns a shared reference to the [`ControlFrame`] at `depth` if any.
    pub fn get(&self, depth: u32) -> &ControlFrame {
        let height = self.height();
        self.frames
            .iter()
            .rev()
            .nth(depth as usize)
            .unwrap_or_else(|| {
                panic!("out of bounds control frame at depth (={depth}) for stack of height (={height})")
            })
    }

    /// Returns an exclusive reference to the [`ControlFrame`] at `depth` if any.
    pub fn get_mut(&mut self, depth: u32) -> &mut ControlFrame {
        let height = self.height();
        self.frames
            .iter_mut()
            .rev()
            .nth(depth as usize)
            .unwrap_or_else(|| {
                panic!("out of bounds control frame at depth (={depth}) for stack of height (={height})")
            })
    }
}

/// An acquired branch target.
#[derive(Debug)]
pub enum AcquiredTarget<'stack> {
    /// The branch targets the function enclosing `block` and therefore is a `return`.
    Return(&'stack mut ControlFrame),
    /// The branch targets a regular [`ControlFrame`].
    Branch(&'stack mut ControlFrame),
}

impl<'stack> AcquiredTarget<'stack> {
    /// Returns an exclusive reference to the [`ControlFrame`] of the [`AcquiredTarget`].
    pub fn control_frame(&'stack mut self) -> &'stack mut ControlFrame {
        match self {
            Self::Return(frame) => frame,
            Self::Branch(frame) => frame,
        }
    }
}

impl ControlStack {
    /// Acquires the target [`ControlFrame`] at the given relative `depth`.
    pub fn acquire_target(&mut self, depth: u32) -> AcquiredTarget {
        let is_root = self.is_root(depth);
        let height = self.height();
        let frame = self.get_mut(depth);
        if is_root {
            AcquiredTarget::Return(frame)
        } else {
            AcquiredTarget::Branch(frame)
        }
    }

    /// Returns `true` if `depth` points to the first control flow frame.
    fn is_root(&self, depth: u32) -> bool {
        if self.frames.is_empty() {
            return false;
        }
        depth as usize == self.height() - 1
    }
}

/// A Wasm control frame.
#[derive(Debug)]
pub struct ControlFrame {
    /// The block type of the [`ControlFrame`].
    ty: BlockType,
    /// The value stack height upon entering the [`ControlFrame`].
    height: usize,
    /// The number of branches to the [`ControlFrame`].
    len_branches: usize,
    /// The [`ControlFrame`]'s [`Instruction::ConsumeFuel`] if fuel metering is enabled.
    ///
    /// # Note
    ///
    /// This is `Some` if fuel metering is enabled and `None` otherwise.
    consume_fuel: Option<Instr>,
    /// The kind of [`ControlFrame`] with associated data.
    kind: ControlFrameKind,
}

impl ControlFrame {
    /// Creates a new unreachable [`ControlFrame`] of `kind`.
    pub fn new_unreachable(ty: BlockType, height: usize, kind: UnreachableControlFrame) -> Self {
        Self {
            ty,
            height,
            len_branches: 0,
            consume_fuel: None,
            kind: ControlFrameKind::Unreachable(kind),
        }
    }

    /// Creates a new Wasm `block` [`ControlFrame`].
    pub fn new_block(
        ty: BlockType,
        height: usize,
        label: LabelRef,
        consume_fuel: Option<Instr>,
    ) -> Self {
        Self {
            ty,
            height,
            len_branches: 0,
            consume_fuel,
            kind: ControlFrameKind::Block(BlockControlFrame { label }),
        }
    }

    /// Creates a new Wasm `loop` [`ControlFrame`].
    pub fn new_loop(
        ty: BlockType,
        height: usize,
        label: LabelRef,
        consume_fuel: Option<Instr>,
    ) -> Self {
        Self {
            ty,
            height,
            len_branches: 0,
            consume_fuel,
            kind: ControlFrameKind::Loop(LoopControlFrame { label }),
        }
    }

    /// Creates a new Wasm `if` [`ControlFrame`].
    pub fn new_if(
        ty: BlockType,
        height: usize,
        label: LabelRef,
        consume_fuel: Option<Instr>,
        reachability: IfReachability,
    ) -> Self {
        Self {
            ty,
            height,
            len_branches: 0,
            consume_fuel,
            kind: ControlFrameKind::If(IfControlFrame {
                label,
                reachability,
            }),
        }
    }

    /// Creates a new Wasm `else` [`ControlFrame`].
    pub fn new_else(
        ty: BlockType,
        height: usize,
        label: LabelRef,
        consume_fuel: Option<Instr>,
        reachability: ElseReachability,
        is_end_of_then_reachable: bool,
    ) -> Self {
        Self {
            ty,
            height,
            len_branches: 0,
            consume_fuel,
            kind: ControlFrameKind::Else(ElseControlFrame {
                label,
                reachability,
                is_end_of_then_reachable,
            }),
        }
    }

    /// Returns the stack height of the [`ControlFrame`].
    pub fn height(&self) -> usize {
        self.height
    }

    /// Returns the [`BlockType`] of the [`ControlFrame`].
    pub fn block_type(&self) -> BlockType {
        self.ty
    }

    /// Returns `true` if at least one branch targets the [`ControlFrame`].
    pub fn is_branched_to(&self) -> bool {
        self.len_branches() >= 1
    }

    /// Returns the number of branches to the [`ControlFrame`].
    fn len_branches(&self) -> usize {
        self.len_branches
    }

    /// Bumps the number of branches to the [`ControlFrame`] by 1.
    fn bump_branches(&mut self) {
        self.len_branches += 1;
    }

    /// Returns a reference to the [`Instruction::ConsumeFuel`] of the [`ControlFrame`] if any.
    ///
    /// Returns `None` if fuel metering is disabled.
    pub fn consume_fuel_instr(&self) -> Option<Instr> {
        self.consume_fuel
    }
}

/// A Wasm control frame kind.
#[derive(Debug)]
pub enum ControlFrameKind {
    /// A Wasm `block` control frame.
    Block(BlockControlFrame),
    /// A Wasm `loop` control frame.
    Loop(LoopControlFrame),
    /// A Wasm `if` control frame, including `then`.
    If(IfControlFrame),
    /// A Wasm `else` control frame, as part of `if`.
    Else(ElseControlFrame),
    /// A generic unreachable Wasm control frame.
    Unreachable(UnreachableControlFrame),
}

/// A Wasm `block` control frame.
#[derive(Debug)]
pub struct BlockControlFrame {
    /// The label used to branch to the [`BlockControlFrame`].
    label: LabelRef,
}

impl BlockControlFrame {
    /// Returns the branch label of the [`BlockControlFrame`].
    pub fn label(&self) -> LabelRef {
        self.label
    }
}

/// A Wasm `loop` control frame.
#[derive(Debug)]
pub struct LoopControlFrame {
    /// The label used to branch to the [`LoopControlFrame`].
    label: LabelRef,
}

impl LoopControlFrame {
    /// Returns the branch label of the [`LoopControlFrame`].
    pub fn label(&self) -> LabelRef {
        self.label
    }
}

/// A Wasm `if` control frame including `then`.
#[derive(Debug)]
pub struct IfControlFrame {
    /// The label used to branch to the [`IfControlFrame`].
    label: LabelRef,
    /// The reachability of the `then` and `else` blocks.
    reachability: IfReachability,
}

impl IfControlFrame {
    /// Returns the branch label of the [`IfControlFrame`].
    pub fn label(&self) -> LabelRef {
        self.label
    }

    /// Returns the label to the `else` branch of the [`IfControlFrame`].
    pub fn else_label(&self) -> Option<LabelRef> {
        match self.reachability {
            IfReachability::Both { else_label } => Some(else_label),
            IfReachability::OnlyThen | IfReachability::OnlyElse => None,
        }
    }

    /// Returns `true` if the `then` branch is reachable.
    ///
    /// # Note
    ///
    /// The `then` branch is unreachable if the `if` condition is a constant `false` value.
    pub fn is_then_reachable(&self) -> bool {
        match self.reachability {
            IfReachability::Both { .. } | IfReachability::OnlyThen => true,
            IfReachability::OnlyElse => false,
        }
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
    /// The label used to branch to the [`ElseControlFrame`].
    label: LabelRef,
    /// The reachability of the `then` and `else` blocks.
    reachability: ElseReachability,
    /// Is `true` if code is reachable when entering the `else` block.
    ///
    /// # Note
    ///
    /// This means that the end of the `then` block was reachable.
    is_end_of_then_reachable: bool,
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
    Both,
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

impl ElseControlFrame {
    /// Returns the branch label of the [`IfControlFrame`].
    pub fn label(&self) -> LabelRef {
        self.label
    }

    /// Returns `true` if the `then` branch is reachable.
    ///
    /// # Note
    ///
    /// The `then` branch is unreachable if the `if` condition is a constant `false` value.
    pub fn is_then_reachable(&self) -> bool {
        match self.reachability {
            ElseReachability::Both { .. } | ElseReachability::OnlyThen => true,
            ElseReachability::OnlyElse => false,
        }
    }

    /// Returns `true` if the `else` branch is reachable.
    ///
    /// # Note
    ///
    /// The `else` branch is unreachable if the `if` condition is a constant `true` value.
    pub fn is_else_reachable(&self) -> bool {
        match self.reachability {
            ElseReachability::Both { .. } | ElseReachability::OnlyElse => true,
            ElseReachability::OnlyThen => false,
        }
    }

    /// Returns `true` if the end of the `then` branch is reachable.
    #[track_caller]
    pub fn is_end_of_then_reachable(&self) -> bool {
        self.is_end_of_then_reachable
    }
}

/// A generic unreachable Wasm control frame.
#[derive(Debug, Copy, Clone)]
pub enum UnreachableControlFrame {
    /// An unreachable Wasm `block` control frame.
    Block,
    /// An unreachable Wasm `loop` control frame.
    Loop,
    /// An unreachable Wasm `if` control frame.
    If,
    /// An unreachable Wasm `else` control frame.
    Else,
}
