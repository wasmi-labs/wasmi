use super::{labels::LabelRef, Instr};
use crate::module::BlockType;

/// A Wasm `block` control flow frame.
#[derive(Debug, Copy, Clone)]
pub struct BlockControlFrame {
    /// The type of the [`BlockControlFrame`].
    block_type: BlockType,
    /// The value stack height upon entering the [`BlockControlFrame`].
    stack_height: u32,
    /// Label representing the end of the [`BlockControlFrame`].
    end_label: LabelRef,
    /// Instruction to consume fuel upon entering the basic block if fuel metering is enabled.
    ///
    /// # Note
    ///
    /// This might be a reference to the consume fuel instruction of the parent
    /// [`ControlFrame`] of the [`BlockControlFrame`].
    consume_fuel: Option<Instr>,
}

impl BlockControlFrame {
    /// Creates a new [`BlockControlFrame`].
    pub fn new(
        block_type: BlockType,
        end_label: LabelRef,
        stack_height: u32,
        consume_fuel: Option<Instr>,
    ) -> Self {
        Self {
            block_type,
            stack_height,
            end_label,
            consume_fuel,
        }
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

    /// Returns a reference to the [`ConsumeFuel`] instruction of the [`BlockControlFrame`] if any.
    ///
    /// Returns `None` if fuel metering is disabled.
    ///
    /// # Note
    ///
    /// A [`BlockControlFrame`] might share its [`ConsumeFuel`] instruction with its child [`BlockControlFrame`].
    ///
    /// [`ConsumeFuel`]: enum.Instruction.html#variant.ConsumeFuel
    pub fn consume_fuel_instr(&self) -> Option<Instr> {
        self.consume_fuel
    }
}

/// A Wasm `loop` control flow frame.
#[derive(Debug, Copy, Clone)]
pub struct LoopControlFrame {
    /// The type of the [`LoopControlFrame`].
    block_type: BlockType,
    /// The value stack height upon entering the [`LoopControlFrame`].
    stack_height: u32,
    /// Label representing the head of the [`LoopControlFrame`].
    head_label: LabelRef,
    /// Instruction to consume fuel upon entering the basic block if fuel metering is enabled.
    ///
    /// # Note
    ///
    /// This must be `Some` if fuel metering is enabled and `None` otherwise.
    consume_fuel: Option<Instr>,
}

impl LoopControlFrame {
    /// Creates a new [`LoopControlFrame`].
    pub fn new(
        block_type: BlockType,
        head_label: LabelRef,
        stack_height: u32,
        consume_fuel: Option<Instr>,
    ) -> Self {
        Self {
            block_type,
            stack_height,
            head_label,
            consume_fuel,
        }
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

    /// Returns a reference to the [`ConsumeFuel`] instruction of the [`BlockControlFrame`] if any.
    ///
    /// Returns `None` if fuel metering is disabled.
    ///
    /// [`ConsumeFuel`]: enum.Instruction.html#variant.ConsumeFuel
    pub fn consume_fuel_instr(&self) -> Option<Instr> {
        self.consume_fuel
    }
}

/// A Wasm `if` and `else` control flow frames.
#[derive(Debug, Copy, Clone)]
pub struct IfControlFrame {
    /// The type of the [`IfControlFrame`].
    block_type: BlockType,
    /// The value stack height upon entering the [`IfControlFrame`].
    stack_height: u32,
    /// Label representing the end of the [`IfControlFrame`].
    end_label: LabelRef,
    /// Label representing the optional `else` branch of the [`IfControlFrame`].
    else_label: LabelRef,
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
    /// Instruction to consume fuel upon entering the basic block if fuel metering is enabled.
    ///
    /// This is used for both `then` and `else` blocks. When entering the `else`
    /// block this field is updated to represent the [`ConsumeFuel`] instruction
    /// of the `else` block instead of the `then` block. This is possible because
    /// only one of them is needed at the same time during translation.
    ///
    /// # Note
    ///
    /// This must be `Some` if fuel metering is enabled and `None` otherwise.
    ///
    /// [`ConsumeFuel`]: enum.Instruction.html#variant.ConsumeFuel
    consume_fuel: Option<Instr>,
}

impl IfControlFrame {
    /// Creates a new [`IfControlFrame`].
    pub fn new(
        block_type: BlockType,
        end_label: LabelRef,
        else_label: LabelRef,
        stack_height: u32,
        consume_fuel: Option<Instr>,
    ) -> Self {
        assert_ne!(
            end_label, else_label,
            "end and else labels must be different"
        );
        Self {
            block_type,
            stack_height,
            end_label,
            else_label,
            end_of_then_is_reachable: None,
            consume_fuel,
        }
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
    pub fn else_label(&self) -> LabelRef {
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

    /// Updates the reachability of the end of the `then` branch.
    ///
    /// # Panics
    ///
    /// If this information has already been provided prior.
    pub fn update_end_of_then_reachability(&mut self, reachable: bool) {
        assert!(self.end_of_then_is_reachable.is_none());
        self.end_of_then_is_reachable = Some(reachable);
    }

    /// Returns a reference to the [`ConsumeFuel`] instruction of the [`BlockControlFrame`] if any.
    ///
    /// Returns `None` if fuel metering is disabled.
    ///
    /// # Note
    ///
    /// This returns the [`ConsumeFuel`] instruction for both `then` and `else` blocks.
    /// When entering the `if` block it represents the [`ConsumeFuel`] instruction until
    /// the `else` block entered. This is possible because only one of them is needed
    /// at the same time during translation.
    ///
    /// [`ConsumeFuel`]: enum.Instruction.html#variant.ConsumeFuel
    pub fn consume_fuel_instr(&self) -> Option<Instr> {
        self.consume_fuel
    }

    /// Updates the [`ConsumeFuel`] instruction for when the `else` block is entered.
    ///
    /// # Note
    ///
    /// This is required since the `consume_fuel` field represents the [`ConsumeFuel`]
    /// instruction for both `then` and `else` blocks. This is possible because only one
    /// of them is needed at the same time during translation.
    ///
    /// [`ConsumeFuel`]: enum.Instruction.html#variant.ConsumeFuel
    pub fn update_consume_fuel_instr(&mut self, instr: Instr) {
        assert!(
            self.consume_fuel.is_some(),
            "can only update the consume fuel instruction if it existed before"
        );
        self.consume_fuel = Some(instr);
    }
}

/// An unreachable control flow frame of any kind.
#[derive(Debug, Copy, Clone)]
pub struct UnreachableControlFrame {
    /// The non-SSA input and output types of the unreachable control frame.
    pub block_type: BlockType,
    /// The kind of the unreachable control flow frame.
    pub kind: ControlFrameKind,
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
    pub fn new(kind: ControlFrameKind, block_type: BlockType) -> Self {
        Self { block_type, kind }
    }

    /// Returns the [`ControlFrameKind`] of the [`UnreachableControlFrame`].
    pub fn kind(&self) -> ControlFrameKind {
        self.kind
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
    /// Returns the [`ControlFrameKind`] of the [`ControlFrame`].
    pub fn kind(&self) -> ControlFrameKind {
        match self {
            ControlFrame::Block(_) => ControlFrameKind::Block,
            ControlFrame::Loop(_) => ControlFrameKind::Loop,
            ControlFrame::If(_) => ControlFrameKind::If,
            ControlFrame::Unreachable(frame) => frame.kind(),
        }
    }

    /// Returns the label for the branch destination of the [`ControlFrame`].
    pub fn branch_destination(&self) -> LabelRef {
        match self {
            Self::Block(frame) => frame.branch_destination(),
            Self::Loop(frame) => frame.branch_destination(),
            Self::If(frame) => frame.branch_destination(),
            Self::Unreachable(frame) => panic!(
                "tried to get `branch_destination` for an unreachable control frame: {frame:?}"
            ),
        }
    }

    /// Returns a label which should be resolved at the `End` Wasm opcode.
    ///
    /// All [`ControlFrame`] kinds have it except [`ControlFrame::Loop`].
    /// In order to a [`ControlFrame::Loop`] to branch outside it is required
    /// to be wrapped in another control frame such as [`ControlFrame::Block`].
    pub fn end_label(&self) -> LabelRef {
        match self {
            Self::Block(frame) => frame.end_label(),
            Self::If(frame) => frame.end_label(),
            Self::Loop(frame) => {
                panic!("tried to get `end_label` for a loop control frame: {frame:?}")
            }
            Self::Unreachable(frame) => {
                panic!("tried to get `end_label` for an unreachable control frame: {frame:?}")
            }
        }
    }

    /// Returns the value stack height upon entering the control flow frame.
    pub fn stack_height(&self) -> Option<u32> {
        match self {
            Self::Block(frame) => Some(frame.stack_height()),
            Self::Loop(frame) => Some(frame.stack_height()),
            Self::If(frame) => Some(frame.stack_height()),
            Self::Unreachable(_frame) => None,
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

    /// Returns `true` if the control flow frame is reachable.
    pub fn is_reachable(&self) -> bool {
        !matches!(self, ControlFrame::Unreachable(_))
    }

    /// Returns a reference to the [`ConsumeFuel`] instruction of the [`ControlFrame`] if any.
    ///
    /// Returns `None` if fuel metering is disabled.
    ///
    /// [`ConsumeFuel`]: enum.Instruction.html#variant.ConsumeFuel
    pub fn consume_fuel_instr(&self) -> Option<Instr> {
        match self {
            ControlFrame::Block(frame) => frame.consume_fuel_instr(),
            ControlFrame::Loop(frame) => frame.consume_fuel_instr(),
            ControlFrame::If(frame) => frame.consume_fuel_instr(),
            ControlFrame::Unreachable(_) => None,
        }
    }
}
