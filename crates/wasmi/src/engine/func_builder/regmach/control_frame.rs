#[cfg(doc)]
use super::ValueStack;
use crate::{
    engine::{
        bytecode2::{RegisterSlice, RegisterSliceIter},
        func_builder::{labels::LabelRef, TranslationErrorInner},
        Instr,
        TranslationError,
    },
    module::BlockType,
    Engine,
};

/// The height of the [`ValueStack`] upon entering the control frame without its parameters.
///
/// # Note
///
/// Used to truncate the [`ValueStack`] after successfully translating a control
/// frame or when encountering unreachable code during its translation.
#[derive(Debug, Default, Copy, Clone)]
pub struct BlockHeight(u16);

impl BlockHeight {
    /// Creates a new [`BlockHeight`] for the given [`ValueStack`] `height` and [`BlockType`].
    pub fn new(
        engine: &Engine,
        height: usize,
        block_type: BlockType,
    ) -> Result<Self, TranslationError> {
        fn new_impl(engine: &Engine, height: usize, block_type: BlockType) -> Option<BlockHeight> {
            let len_params = u16::try_from(block_type.len_params(engine)).ok()?;
            let height = u16::try_from(height).ok()?;
            let block_height = height.checked_sub(len_params)?;
            Some(Self(block_height))
        }
        new_impl(engine, height, block_type)
            .ok_or_else(|| TranslationErrorInner::EmulatedValueStackOverflow)
            .map_err(TranslationError::new)
    }

    /// Returns the `u16` value of the [`BlockHeight`].
    pub fn into_u16(self) -> u16 {
        self.0
    }
}

/// A Wasm `block` control flow frame.
#[derive(Debug, Copy, Clone)]
pub struct BlockControlFrame {
    /// The type of the [`BlockControlFrame`].
    block_type: BlockType,
    /// The number of branches to this [`BlockControlFrame`].
    len_branches: usize,
    /// The value stack height upon entering the [`BlockControlFrame`].
    stack_height: BlockHeight,
    /// Label representing the end of the [`BlockControlFrame`].
    end_label: LabelRef,
    /// The branch parameters of the [`BlockControlFrame`].
    ///
    /// # Note
    ///
    /// These are the registers that store the results of
    /// the [`BlockControlFrame`] upon taking a branch to it.
    /// Note that branching to a [`BlockControlFrame`] exits it.
    branch_params: RegisterSlice,
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
        branch_params: RegisterSlice,
        stack_height: BlockHeight,
        consume_fuel: Option<Instr>,
    ) -> Self {
        Self {
            block_type,
            len_branches: 0,
            stack_height,
            end_label,
            branch_params,
            consume_fuel,
        }
    }

    /// Returns `true` if at least one branch targets this [`BlockControlFrame`].
    pub fn is_branched_to(&self) -> bool {
        self.len_branches() >= 1
    }

    /// Returns the number of branches to this [`BlockControlFrame`].
    fn len_branches(&self) -> usize {
        self.len_branches
    }

    /// Bumps the number of branches to this [`BlockControlFrame`] by 1.
    fn bump_branches(&mut self) {
        self.len_branches += 1;
    }

    /// Returns an iterator over the registers holding the branching parameters of the [`BlockControlFrame`].
    pub fn branch_params(&self, engine: &Engine) -> RegisterSliceIter {
        self.branch_params
            .iter(self.block_type().len_results(engine) as usize)
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

    /// Returns the [`BlockHeight`] of the [`BlockControlFrame`].
    pub fn block_height(&self) -> BlockHeight {
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
    /// The number of branches to this [`BlockControlFrame`].
    len_branches: usize,
    /// The value stack height upon entering the [`LoopControlFrame`].
    stack_height: BlockHeight,
    /// Label representing the head of the [`LoopControlFrame`].
    head_label: LabelRef,
    /// The branch parameters of the [`LoopControlFrame`].
    ///
    /// # Note
    ///
    /// These are the registers that store the inputs of
    /// the [`LoopControlFrame`] upon taking a branch to it.
    /// Note that branching to a [`LoopControlFrame`] re-enters it.
    branch_params: RegisterSlice,
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
        stack_height: BlockHeight,
        branch_params: RegisterSlice,
        consume_fuel: Option<Instr>,
    ) -> Self {
        Self {
            block_type,
            len_branches: 0,
            stack_height,
            head_label,
            branch_params,
            consume_fuel,
        }
    }

    /// Returns `true` if at least one branch targets this [`LoopControlFrame`].
    pub fn is_branched_to(&self) -> bool {
        self.len_branches() >= 1
    }

    /// Returns the number of branches to this [`LoopControlFrame`].
    fn len_branches(&self) -> usize {
        self.len_branches
    }

    /// Bumps the number of branches to this [`LoopControlFrame`] by 1.
    fn bump_branches(&mut self) {
        self.len_branches += 1;
    }

    /// Returns an iterator over the registers holding the branching parameters of the [`LoopControlFrame`].
    pub fn branch_params(&self, engine: &Engine) -> RegisterSliceIter {
        self.branch_params
            .iter(self.block_type().len_params(engine) as usize)
    }

    /// Returns the label for the branch destination of the [`LoopControlFrame`].
    ///
    /// # Note
    ///
    /// Branches to [`LoopControlFrame`] jump to the head of the loop.
    pub fn branch_destination(&self) -> LabelRef {
        self.head_label
    }

    /// Returns the [`BlockHeight`] of the [`LoopControlFrame`].
    pub fn block_height(&self) -> BlockHeight {
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
    /// The number of branches to this [`BlockControlFrame`].
    len_branches: usize,
    /// The value stack height upon entering the [`IfControlFrame`].
    stack_height: BlockHeight,
    /// Label representing the end of the [`IfControlFrame`].
    end_label: LabelRef,
    /// Label representing the optional `else` branch of the [`IfControlFrame`].
    else_label: LabelRef,
    /// The branch parameters of the [`IfControlFrame`].
    ///
    /// # Note
    ///
    /// These are the registers that store the results of
    /// the [`IfControlFrame`] upon taking a branch to it.
    /// Note that branching to a [`IfControlFrame`] exits it.
    /// The behavior is the same for the `then` and `else` blocks.
    branch_params: RegisterSlice,
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
        stack_height: BlockHeight,
        branch_params: RegisterSlice,
        consume_fuel: Option<Instr>,
    ) -> Self {
        assert_ne!(
            end_label, else_label,
            "end and else labels must be different"
        );
        Self {
            block_type,
            len_branches: 0,
            stack_height,
            end_label,
            else_label,
            branch_params,
            end_of_then_is_reachable: None,
            consume_fuel,
        }
    }

    /// Returns `true` if at least one branch targets this [`IfControlFrame`].
    pub fn is_branched_to(&self) -> bool {
        self.len_branches() >= 1
    }

    /// Returns the number of branches to this [`IfControlFrame`].
    fn len_branches(&self) -> usize {
        self.len_branches
    }

    /// Bumps the number of branches to this [`IfControlFrame`] by 1.
    fn bump_branches(&mut self) {
        self.len_branches += 1;
    }

    /// Returns an iterator over the registers holding the branching parameters of the [`IfControlFrame`].
    pub fn branch_params(&self, engine: &Engine) -> RegisterSliceIter {
        self.branch_params
            .iter(self.block_type().len_results(engine) as usize)
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

    /// Returns the [`BlockHeight`] of the [`IfControlFrame`].
    pub fn block_height(&self) -> BlockHeight {
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

    /// Returns an iterator over the registers holding the branch parameters of the [`ControlFrame`].
    pub fn branch_params(&self, engine: &Engine) -> RegisterSliceIter {
        match self {
            Self::Block(frame) => frame.branch_params(engine),
            Self::Loop(frame) => frame.branch_params(engine),
            Self::If(frame) => frame.branch_params(engine),
            Self::Unreachable(frame) => {
                panic!("tried to get `branch_params` for an unreachable control frame: {frame:?}")
            }
        }
    }

    /// Returns the label for the branch destination of the [`ControlFrame`].
    pub fn branch_destination(&self) -> LabelRef {
        match self {
            Self::Block(frame) => frame.branch_destination(),
            Self::Loop(frame) => frame.branch_destination(),
            Self::If(frame) => frame.branch_destination(),
            Self::Unreachable(frame) => panic!(
                "tried to call `branch_destination` for an unreachable control frame: {frame:?}"
            ),
        }
    }

    /// Returns `true` if at least one branch targets this [`ControlFrame`].
    pub fn is_branched_to(&self) -> bool {
        match self {
            Self::Block(frame) => frame.is_branched_to(),
            Self::Loop(frame) => frame.is_branched_to(),
            Self::If(frame) => frame.is_branched_to(),
            Self::Unreachable(frame) => {
                panic!("tried to call `is_branched_to` for an unreachable control frame: {frame:?}")
            }
        }
    }

    /// Returns the number of branches to the [`ControlFrame`].
    fn len_branches(&self) -> usize {
        match self {
            Self::Block(frame) => frame.len_branches(),
            Self::Loop(frame) => frame.len_branches(),
            Self::If(frame) => frame.len_branches(),
            Self::Unreachable(frame) => {
                panic!("tried to call `len_branches` for an unreachable control frame: {frame:?}")
            }
        }
    }

    /// Bumps the number of branches to this [`ControlFrame`] by 1.
    fn bump_branches(&mut self) {
        match self {
            ControlFrame::Block(frame) => frame.bump_branches(),
            ControlFrame::Loop(frame) => frame.bump_branches(),
            ControlFrame::If(frame) => frame.bump_branches(),
            Self::Unreachable(frame) => {
                panic!("tried to `bump_branches` on an unreachable control frame: {frame:?}")
            }
        }
    }

    /// Returns a label which should be resolved at the `End` Wasm opcode.
    ///
    /// # Note
    ///
    /// The [`LoopControlFrame`] does not have an `end_label` since all
    /// branches targeting it are branching to the loop header instead.
    /// Exiting a [`LoopControlFrame`] is simply done by leaving its scope
    /// or branching to a parent [`ControlFrame`].
    pub fn end_label(&self) -> Option<LabelRef> {
        match self {
            Self::Block(frame) => Some(frame.end_label()),
            Self::If(frame) => Some(frame.end_label()),
            Self::Loop(_frame) => None,
            Self::Unreachable(_frame) => None,
        }
    }

    /// Returns the [`BlockHeight`] upon entering the control flow frame.
    ///
    /// # Note
    ///
    /// The [`UnreachableControlFrame`] does not need or have a [`BlockHeight`].
    pub fn block_height(&self) -> Option<BlockHeight> {
        match self {
            Self::Block(frame) => Some(frame.block_height()),
            Self::Loop(frame) => Some(frame.block_height()),
            Self::If(frame) => Some(frame.block_height()),
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
