use crate::{engine::LabelIdx, module2::BlockType};

/// A Wasm `block` control flow frame.
#[derive(Debug, Copy, Clone)]
pub struct BlockControlFrame {
    /// Label representing the end of the [`BlockControlFrame`].
    end_label: LabelIdx,
    /// The type of the [`BlockControlFrame`].
    block_type: BlockType,
    /// The value stack height upon entering the [`BlockControlFrame`].
    stack_height: u32,
}

impl BlockControlFrame {
    /// Creates a new [`BlockControlFrame`].
    pub fn new(block_type: BlockType, end_label: LabelIdx, stack_height: u32) -> Self {
        Self {
            block_type,
            end_label,
            stack_height,
        }
    }

    /// Returns the label for the branch destination of the [`BlockControlFrame`].
    ///
    /// # Note
    ///
    /// Branches to [`BlockControlFrame`] jump to the end of the frame.
    pub fn branch_destination(&self) -> LabelIdx {
        self.end_label
    }

    /// Returns the label to the end of the [`BlockControlFrame`].
    pub fn end_label(&self) -> LabelIdx {
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
    /// Label representing the head of the [`LoopControlFrame`].
    head_label: LabelIdx,
    /// The type of the [`LoopControlFrame`].
    block_type: BlockType,
    /// The value stack height upon entering the [`LoopControlFrame`].
    stack_height: u32,
}

impl LoopControlFrame {
    /// Creates a new [`LoopControlFrame`].
    pub fn new(block_type: BlockType, head_label: LabelIdx, stack_height: u32) -> Self {
        Self {
            block_type,
            head_label,
            stack_height,
        }
    }

    /// Returns the label for the branch destination of the [`LoopControlFrame`].
    ///
    /// # Note
    ///
    /// Branches to [`LoopControlFrame`] jump to the head of the loop.
    pub fn branch_destination(&self) -> LabelIdx {
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
    /// Label representing the end of the [`IfControlFrame`].
    end_label: LabelIdx,
    /// Label representing the optional `else` branch of the [`IfControlFrame`].
    else_label: LabelIdx,
    /// The type of the [`IfControlFrame`].
    block_type: BlockType,
    /// The value stack height upon entering the [`IfControlFrame`].
    stack_height: u32,
}

impl IfControlFrame {
    /// Creates a new [`IfControlFrame`].
    pub fn new(
        block_type: BlockType,
        end_label: LabelIdx,
        else_label: LabelIdx,
        stack_height: u32,
    ) -> Self {
        assert_ne!(
            end_label, else_label,
            "end and else labels must be different"
        );
        Self {
            block_type,
            end_label,
            else_label,
            stack_height,
        }
    }

    /// Returns the label for the branch destination of the [`IfControlFrame`].
    ///
    /// # Note
    ///
    /// Branches to [`IfControlFrame`] jump to the end of the if and else frame.
    pub fn branch_destination(&self) -> LabelIdx {
        self.end_label
    }

    /// Returns the label to the end of the [`IfControlFrame`].
    pub fn end_label(&self) -> LabelIdx {
        self.end_label
    }

    /// Returns the label to the optional `else` of the [`IfControlFrame`].
    pub fn else_label(&self) -> LabelIdx {
        self.else_label
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
    pub fn new(block_type: BlockType, kind: ControlFrameKind, stack_height: u32) -> Self {
        Self {
            block_type,
            kind,
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
    pub fn branch_destination(&self) -> LabelIdx {
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

    /// Returns a label which should be resolved at the `End` Wasm opcode.
    ///
    /// All [`ControlFrame`] kinds have it except [`ControlFrame::Loop`].
    /// In order to a [`ControlFrame::Loop`] to branch outside it is required
    /// to be wrapped in another control frame such as [`ControlFrame::Block`].
    pub fn end_label(&self) -> LabelIdx {
        match self {
            Self::Block(frame) => frame.end_label(),
            Self::If(frame) => frame.end_label(),
            Self::Loop(frame) => panic!(
                "tried to get `end_label` for a loop control frame: {:?}",
                frame
            ),
            Self::Unreachable(frame) => panic!(
                "tried to get `end_label` for an unreachable control frame: {:?}",
                frame
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
}
