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

/// A control flow frame.
#[derive(Debug, Copy, Clone)]
pub enum ControlFrame {
    /// Basic block control frame.
    Block(BlockControlFrame),
    /// Loop control frame.
    Loop(LoopControlFrame),
    /// If and else control frame.
    If(IfControlFrame),
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

impl ControlFrame {
    /// Returns the label for the branch destination of the [`ControlFrame`].
    pub fn branch_destination(&self) -> LabelIdx {
        match self {
            Self::Block(block_frame) => block_frame.branch_destination(),
            Self::Loop(loop_frame) => loop_frame.branch_destination(),
            Self::If(if_frame) => if_frame.branch_destination(),
        }
    }

    /// Returns a label which should be resolved at the `End` Wasm opcode.
    ///
    /// All [`ControlFrame`] kinds have it except [`ControlFrame::Loop`].
    /// In order to a [`ControlFrame::Loop`] to branch outside it is required
    /// to be wrapped in another control frame such as [`ControlFrame::Block`].
    pub fn end_label(&self) -> LabelIdx {
        match self {
            Self::Block(block_frame) => block_frame.end_label(),
            Self::If(if_frame) => if_frame.end_label(),
            Self::Loop(loop_frame) => panic!(
                "tried to receive `end_label` which is not supported for loop control frames: {:?}",
                loop_frame
            ),
        }
    }

    /// Returns the value stack height upon entering the control flow frame.
    pub fn stack_height(&self) -> u32 {
        match self {
            Self::Block(block_frame) => block_frame.stack_height(),
            Self::Loop(loop_frame) => loop_frame.stack_height(),
            Self::If(if_frame) => if_frame.stack_height(),
        }
    }

    /// Returns the [`BlockType`] of the control flow frame.
    pub fn block_type(&self) -> BlockType {
        match self {
            Self::Block(block_frame) => block_frame.block_type(),
            Self::Loop(loop_frame) => loop_frame.block_type(),
            Self::If(if_frame) => if_frame.block_type(),
        }
    }
}
