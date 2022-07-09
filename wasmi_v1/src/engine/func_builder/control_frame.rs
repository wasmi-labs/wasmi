use super::{IrRegisterSlice, LabelRef};
use crate::module::BlockType;

/// A Wasm `block` control flow frame.
#[derive(Debug, Copy, Clone)]
pub struct BlockControlFrame {
    /// The registers holding the results of the [`BlockControlFrame`].
    results: IrRegisterSlice,
    /// Label representing the end of the [`BlockControlFrame`].
    end_label: LabelRef,
    /// The type of the [`BlockControlFrame`].
    block_type: BlockType,
    /// The value stack height upon entering the [`BlockControlFrame`].
    stack_height: u32,
}

impl BlockControlFrame {
    /// Creates a new [`BlockControlFrame`].
    pub fn new(
        results: IrRegisterSlice,
        block_type: BlockType,
        end_label: LabelRef,
        stack_height: u32,
    ) -> Self {
        Self {
            results,
            block_type,
            end_label,
            stack_height,
        }
    }

    /// Returns the [`IrRegisterSlice`] to put the results of the [`BlockControlFrame`].
    ///
    /// # Note
    ///
    /// This is used when branching to this [`BlockControlFrame`].
    pub fn branch_results(&self) -> IrRegisterSlice {
        self.results
    }

    /// Returns the [`IrRegisterSlice`] to put the results of the [`BlockControlFrame`].
    ///
    /// # Note
    ///
    /// This is used when ending this [`BlockControlFrame`].
    pub fn end_results(&self) -> IrRegisterSlice {
        self.results
    }

    /// Returns the label for the branch destination of the [`BlockControlFrame`].
    ///
    /// # Note
    ///
    /// Branches to [`BlockControlFrame`] jump to the end of the frame.
    pub fn branch_destination(&self) -> LabelRef {
        self.end_label
    }

    /// Returns the label to the end of the [`BlockControlFrame`].
    pub fn end_label(&self) -> LabelRef {
        self.end_label
    }

    /// Returns the value stack height upon entering the [`BlockControlFrame`].
    pub fn stack_height(&self) -> u32 {
        self.stack_height
    }

    /// Returns the [`BlockType`] of the [`BlockControlFrame`].
    pub fn block_type(&self) -> BlockType {
        self.block_type
    }
}

/// A Wasm `loop` control flow frame.
#[derive(Debug, Copy, Clone)]
pub struct LoopControlFrame {
    /// The registers holding the results of the [`LoopControlFrame`].
    branch_results: IrRegisterSlice,
    /// The registers holding the results of the [`LoopControlFrame`].
    end_results: IrRegisterSlice,
    /// Label representing the head of the [`LoopControlFrame`].
    head_label: LabelRef,
    /// The type of the [`LoopControlFrame`].
    block_type: BlockType,
    /// The value stack height upon entering the [`LoopControlFrame`].
    stack_height: u32,
}

impl LoopControlFrame {
    /// Creates a new [`LoopControlFrame`].
    pub fn new(
        branch_results: IrRegisterSlice,
        end_results: IrRegisterSlice,
        block_type: BlockType,
        head_label: LabelRef,
        stack_height: u32,
    ) -> Self {
        Self {
            branch_results,
            end_results,
            block_type,
            head_label,
            stack_height,
        }
    }

    /// Returns the [`IrRegisterSlice`] to put the results of the [`LoopControlFrame`].
    ///
    /// # Note
    ///
    /// This is used when branching to this [`LoopControlFrame`].
    pub fn branch_results(&self) -> IrRegisterSlice {
        self.branch_results
    }

    /// Returns the [`IrRegisterSlice`] to put the results of the [`LoopControlFrame`].
    ///
    /// # Note
    ///
    /// This is used when ending this [`LoopControlFrame`].
    pub fn end_results(&self) -> IrRegisterSlice {
        self.end_results
    }

    /// Returns the label for the branch destination of the [`LoopControlFrame`].
    ///
    /// # Note
    ///
    /// Branches to [`LoopControlFrame`] jump to the head of the loop.
    pub fn branch_destination(&self) -> LabelRef {
        self.head_label
    }

    /// Returns the value stack height upon entering the [`LoopControlFrame`].
    pub fn stack_height(&self) -> u32 {
        self.stack_height
    }

    /// Returns the [`BlockType`] of the [`LoopControlFrame`].
    pub fn block_type(&self) -> BlockType {
        self.block_type
    }
}

/// A Wasm `if` and `else` control flow frames.
#[derive(Debug, Copy, Clone)]
pub struct IfControlFrame {
    /// The registers holding the results of the [`IfControlFrame`].
    results: IrRegisterSlice,
    /// Label representing the end of the [`IfControlFrame`].
    end_label: LabelRef,
    /// Label representing the optional `else` branch of the [`IfControlFrame`].
    ///
    /// # Note
    ///
    /// This might be `None` in case it is known at compile time that the
    /// `else` block is unreachable while the `then` block is reachable.
    else_label: Option<LabelRef>,
    /// The type of the [`IfControlFrame`].
    block_type: BlockType,
    /// The provider stack height upon entering the [`IfControlFrame`].
    stack_height: u32,
    /// The provider stack height for the otional `else` block.
    ///
    /// # Note
    ///
    /// Upon entering an `if` block we duplicate the parameters for the `if`
    /// block so that we can reuse the provider stack for both, the parameters
    /// of the `if` block as well as for the parameters of the optional `else`
    /// block.
    /// This way we do not have to store the block parameters out of space,
    /// for example in this structure which would not be as efficient.
    else_height: Option<u32>,
    /// End of `then` branch is reachable.
    ///
    /// # Note
    ///
    /// - This is `None` upon entering the `if` control flow frame.
    ///   Once the optional `else` case or the `end` of the `if` control
    ///   flow frame is reached this field will be computed.
    /// - This information is important to know how to continue after a
    ///   diverging `if` control flow frame.
    /// - An `end_of_else_is_reachable` field is not needed since it will
    ///   be easily computed once the translation reaches the end of the `if`.
    end_of_then_is_reachable: Option<bool>,
    /// The reachability of the `if` and its `then` and `else` blocks.
    reachability: IfReachability,
}

/// The reachability of the `if` control flow frame.
#[derive(Debug, Copy, Clone)]
pub enum IfReachability {
    /// Both, `then` and `else` blocks of the `if` are reachable.
    Both,
    /// Only the `then` block of the `if` is reachable.
    ///
    /// # Note
    ///
    /// This might be `false` if the condition of the `if` is a constant `false`
    /// value. In this case the `wasmi` translator flattens the `if` block to
    /// the `else` case.
    OnlyThen,
    /// Only the `else` block of the `if` is reachable.
    ///
    /// # Note
    ///
    /// This might be `false` if the condition of the `if` is a constant `true`
    /// value. In this case the `wasmi` translator flattens the `if` block to
    /// the `then` case.
    OnlyElse,
}

impl IfControlFrame {
    /// Creates a new [`IfControlFrame`].
    pub fn new(
        results: IrRegisterSlice,
        block_type: BlockType,
        end_label: LabelRef,
        else_label: Option<LabelRef>,
        stack_height: u32,
        else_height: Option<u32>,
        reachability: IfReachability,
    ) -> Self {
        assert_ne!(
            Some(end_label),
            else_label,
            "end and else labels must be different"
        );
        Self {
            results,
            block_type,
            end_label,
            else_label,
            stack_height,
            else_height,
            end_of_then_is_reachable: None,
            reachability,
        }
    }

    /// Returns the [`IrRegisterSlice`] to put the results of the [`IfControlFrame`].
    ///
    /// # Note
    ///
    /// This is used when branching to this [`IfControlFrame`].
    pub fn branch_results(&self) -> IrRegisterSlice {
        self.results
    }

    /// Returns the [`IrRegisterSlice`] to put the results of the [`IfControlFrame`].
    ///
    /// # Note
    ///
    /// This is used when ending this [`IfControlFrame`].
    pub fn end_results(&self) -> IrRegisterSlice {
        self.results
    }

    /// Returns the label for the branch destination of the [`IfControlFrame`].
    ///
    /// # Note
    ///
    /// Branches to [`IfControlFrame`] jump to the end of the if and else frame.
    pub fn branch_destination(&self) -> LabelRef {
        self.end_label
    }

    /// Returns the label to the end of the [`IfControlFrame`].
    pub fn end_label(&self) -> LabelRef {
        self.end_label
    }

    /// Returns the label to the optional `else` of the [`IfControlFrame`].
    ///
    /// # Note
    ///
    /// This might be `None` in case it is known at compile time that the
    /// `else` block is unreachable while the `then` block is reachable.
    pub fn else_label(&self) -> Option<LabelRef> {
        self.else_label
    }

    /// Returns the value stack height upon entering the [`IfControlFrame`].
    pub fn stack_height(&self) -> u32 {
        self.stack_height
    }

    /// Returns the value stack height prepared for the `else` of the [`IfControlFrame`].
    ///
    /// # Note
    ///
    /// This might be `None` in case it is known at compile time that the
    /// `else` block is unreachable while the `then` block is reachable.
    pub fn else_height(&self) -> Option<u32> {
        self.else_height
    }

    /// Returns the [`BlockType`] of the [`IfControlFrame`].
    pub fn block_type(&self) -> BlockType {
        self.block_type
    }

    /// Updates the reachability of the end of the `then` branch.
    ///
    /// # Note
    ///
    /// This is guaranteed to be called when visiting the `else` block
    /// of an `if` block. So after visiting the `else` block the
    /// `end_of_then_is_reachable` is always `Some(_)`.
    ///
    /// # Panics
    ///
    /// If this information has already been provided prior.
    pub fn update_end_of_then_reachability(&mut self, reachable: bool) {
        assert!(self.end_of_then_is_reachable.is_none());
        self.end_of_then_is_reachable = Some(reachable);
    }

    /// Returns `true` if the `else` block has been visited.
    pub fn visited_else(&self) -> bool {
        self.end_of_then_is_reachable.is_some()
    }

    /// Returns `true` if the `then` block is known to be reachable.
    pub fn is_then_reachable(&self) -> bool {
        matches!(
            self.reachability,
            IfReachability::Both | IfReachability::OnlyThen
        )
    }

    /// Returns `true` if the `else` block is known to be reachable.
    pub fn is_else_reachable(&self) -> bool {
        matches!(
            self.reachability,
            IfReachability::Both | IfReachability::OnlyElse
        )
    }
}

/// An unreachable control flow frame of any kind.
#[derive(Debug, Copy, Clone)]
pub struct UnreachableControlFrame {
    /// The non-SSA input and output types of the unreachable control frame.
    pub block_type: BlockType,
    /// The kind of the unreachable control flow frame.
    pub kind: ControlFrameKind,
    /// The value stack size upon entering the unreachable control frame.
    pub stack_height: u32,
}

/// The kind of a control flow frame.
#[derive(Debug, Copy, Clone)]
pub enum ControlFrameKind {
    /// A basic `block` control flow frame.
    Block,
    /// A `loop` control flow frame.
    Loop,
    /// An `if` and `else` block control flow frame.
    If,
}

impl UnreachableControlFrame {
    /// Creates a new [`UnreachableControlFrame`] with the given type and kind.
    pub fn new(kind: ControlFrameKind, block_type: BlockType, stack_height: u32) -> Self {
        Self {
            kind,
            block_type,
            stack_height,
        }
    }

    /// Returns the [`ControlFrameKind`] of the [`UnreachableControlFrame`].
    pub fn kind(&self) -> ControlFrameKind {
        self.kind
    }

    /// Returns the value stack height upon entering the [`IfControlFrame`].
    pub fn stack_height(&self) -> u32 {
        self.stack_height
    }

    /// Returns the [`BlockType`] of the [`IfControlFrame`].
    pub fn block_type(&self) -> BlockType {
        self.block_type
    }
}

/// A control flow frame.
#[derive(Debug, Copy, Clone)]
pub enum ControlFrame {
    /// Basic block control frame.
    Block(BlockControlFrame),
    /// Loop control frame.
    Loop(LoopControlFrame),
    /// If and else control frame.
    If(IfControlFrame),
    /// An unreachable control frame.
    Unreachable(UnreachableControlFrame),
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

impl From<UnreachableControlFrame> for ControlFrame {
    fn from(frame: UnreachableControlFrame) -> Self {
        Self::Unreachable(frame)
    }
}

impl ControlFrame {
    /// Returns the label for the branch destination of the [`ControlFrame`].
    pub fn branch_destination(&self) -> LabelRef {
        match self {
            Self::Block(frame) => frame.branch_destination(),
            Self::Loop(frame) => frame.branch_destination(),
            Self::If(frame) => frame.branch_destination(),
            Self::Unreachable(frame) => panic!(
                "tried to get `branch_destination` for an unreachable control frame: {:?}",
                frame,
            ),
        }
    }

    /// Returns the value stack height upon entering the control flow frame.
    pub fn stack_height(&self) -> u32 {
        match self {
            Self::Block(frame) => frame.stack_height(),
            Self::Loop(frame) => frame.stack_height(),
            Self::If(frame) => frame.stack_height(),
            Self::Unreachable(frame) => frame.stack_height(),
        }
    }

    /// Returns the [`BlockType`] of the control flow frame.
    pub fn block_type(&self) -> BlockType {
        match self {
            Self::Block(frame) => frame.block_type(),
            Self::Loop(frame) => frame.block_type(),
            Self::If(frame) => frame.block_type(),
            Self::Unreachable(frame) => frame.block_type(),
        }
    }

    /// Returns the [`IrRegisterSlice`] for where to put
    /// the results of the control flow frame after a `branch` to it.
    pub fn branch_results(&self) -> IrRegisterSlice {
        match self {
            ControlFrame::Block(frame) => frame.branch_results(),
            ControlFrame::Loop(frame) => frame.branch_results(),
            ControlFrame::If(frame) => frame.branch_results(),
            ControlFrame::Unreachable(frame) => panic!(
                "tried to get `branch_results` for an unreachable control frame: {:?}",
                frame
            ),
        }
    }
}
