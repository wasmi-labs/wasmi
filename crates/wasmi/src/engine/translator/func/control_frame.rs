use super::LabelRef;
use crate::{
    engine::{translator::Instr, BlockType, TranslationError},
    ir::{BoundedRegSpan, RegSpan},
    Engine,
    Error,
};

#[cfg(doc)]
use super::ValueStack;

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
    pub fn new(engine: &Engine, height: usize, block_type: BlockType) -> Result<Self, Error> {
        fn new_impl(engine: &Engine, height: usize, block_type: BlockType) -> Option<BlockHeight> {
            let len_params = block_type.len_params(engine);
            let height = u16::try_from(height).ok()?;
            let block_height = height.checked_sub(len_params)?;
            Some(BlockHeight(block_height))
        }
        new_impl(engine, height, block_type)
            .ok_or(TranslationError::EmulatedValueStackOverflow)
            .map_err(Error::from)
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
    /// This is `true` if there is at leat one branch to the [`BlockControlFrame`].
    is_branched_to: bool,
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
    branch_params: RegSpan,
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
        branch_params: RegSpan,
        stack_height: BlockHeight,
        consume_fuel: Option<Instr>,
    ) -> Self {
        Self {
            block_type,
            is_branched_to: false,
            stack_height,
            end_label,
            branch_params,
            consume_fuel,
        }
    }

    /// Returns `true` if at least one branch targets this [`BlockControlFrame`].
    pub fn is_branched_to(&self) -> bool {
        self.is_branched_to
    }

    /// Makes the [`BlockControlFrame`] aware that there is a branch to it.
    fn branch_to(&mut self) {
        self.is_branched_to = true;
    }

    /// Returns an iterator over the registers holding the branching parameters of the [`BlockControlFrame`].
    pub fn branch_params(&self, engine: &Engine) -> BoundedRegSpan {
        BoundedRegSpan::new(self.branch_params, self.block_type().len_results(engine))
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
    /// This is `true` if there is at leat one branch to the [`LoopControlFrame`].
    is_branched_to: bool,
    /// Label representing the head of the [`LoopControlFrame`].
    head_label: LabelRef,
    /// The branch parameters of the [`LoopControlFrame`].
    ///
    /// # Note
    ///
    /// These are the registers that store the inputs of
    /// the [`LoopControlFrame`] upon taking a branch to it.
    /// Note that branching to a [`LoopControlFrame`] re-enters it.
    branch_params: RegSpan,
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
        branch_params: RegSpan,
        consume_fuel: Option<Instr>,
    ) -> Self {
        Self {
            block_type,
            is_branched_to: false,
            head_label,
            branch_params,
            consume_fuel,
        }
    }

    /// Makes the [`BlockControlFrame`] aware that there is a branch to it.
    fn branch_to(&mut self) {
        self.is_branched_to = true;
    }

    /// Returns an iterator over the registers holding the branching parameters of the [`LoopControlFrame`].
    pub fn branch_params(&self, engine: &Engine) -> BoundedRegSpan {
        BoundedRegSpan::new(self.branch_params, self.block_type().len_params(engine))
    }

    /// Returns the label for the branch destination of the [`LoopControlFrame`].
    ///
    /// # Note
    ///
    /// Branches to [`LoopControlFrame`] jump to the head of the loop.
    pub fn branch_destination(&self) -> LabelRef {
        self.head_label
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
    /// This is `true` if there is at leat one branch to the [`IfControlFrame`].
    is_branched_to: bool,
    /// The value stack height upon entering the [`IfControlFrame`].
    stack_height: BlockHeight,
    /// Label representing the end of the [`IfControlFrame`].
    end_label: LabelRef,
    /// The branch parameters of the [`IfControlFrame`].
    ///
    /// # Note
    ///
    /// These are the registers that store the results of
    /// the [`IfControlFrame`] upon taking a branch to it.
    /// Note that branching to a [`IfControlFrame`] exits it.
    /// The behavior is the same for the `then` and `else` blocks.
    branch_params: RegSpan,
    /// Instruction to consume fuel upon entering the basic block if fuel metering is enabled.
    ///
    /// This is used for both `then` and `else` branches. When entering the `else`
    /// block this field is updated to represent the [`ConsumeFuel`] of the
    /// `else` branch instead of the `then` branch. This is possible because
    /// only one of them is needed at the same time during translation.
    ///
    /// # Note
    ///
    /// - This must be `Some` if fuel metering is enabled and `None` otherwise.
    /// - An `if` control frame only needs its own [`ConsumeFuel`] instruction if
    ///   both `then` and `else` branches are reachable. Otherwise we inherit the
    ///   [`ConsumeFuel`] instruction from the parent control frame as we do for
    ///   `block` control frames.
    ///
    /// [`ConsumeFuel`]: enum.Instruction.html#variant.ConsumeFuel
    consume_fuel: Option<Instr>,
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
    /// The reachability of the `then` and `else` blocks of the [`IfControlFrame`].
    reachability: IfReachability,
    /// Indicates whether the `else` block of the [`IfControlFrame`] has been seen already.
    visited_else: bool,
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

impl IfReachability {
    /// Creates an [`IfReachability`] when both `then` and `else` parts are reachable.
    pub fn both(else_label: LabelRef) -> Self {
        Self::Both { else_label }
    }
}

impl IfControlFrame {
    /// Creates a new [`IfControlFrame`].
    pub fn new(
        block_type: BlockType,
        end_label: LabelRef,
        branch_params: RegSpan,
        stack_height: BlockHeight,
        consume_fuel: Option<Instr>,
        reachability: IfReachability,
    ) -> Self {
        if let IfReachability::Both { else_label } = reachability {
            assert_ne!(
                end_label, else_label,
                "end and else labels must be different"
            );
        }
        let end_of_then_is_reachable = match reachability {
            IfReachability::Both { .. } | IfReachability::OnlyThen => None,
            IfReachability::OnlyElse => Some(false),
        };
        Self {
            block_type,
            is_branched_to: false,
            stack_height,
            end_label,
            branch_params,
            consume_fuel,
            end_of_then_is_reachable,
            reachability,
            visited_else: false,
        }
    }

    /// Returns `true` if at least one branch targets this [`IfControlFrame`].
    pub fn is_branched_to(&self) -> bool {
        self.is_branched_to
    }

    /// Makes the [`IfControlFrame`] aware that there is a branch to it.
    pub fn branch_to(&mut self) {
        self.is_branched_to = true;
    }

    /// Returns an iterator over the registers holding the branching parameters of the [`IfControlFrame`].
    pub fn branch_params(&self, engine: &Engine) -> BoundedRegSpan {
        BoundedRegSpan::new(self.branch_params, self.block_type().len_results(engine))
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

    /// Updates the reachability of the end of the `then` branch.
    ///
    /// # Note
    ///
    /// This is expected to be called when visiting the `else` of an
    /// `if` control frame to inform the `if` control frame if the
    /// end of the `then` block is reachable. This information is
    /// important to decide whether code coming after the entire `if`
    /// control frame is reachable again.
    ///
    /// # Panics
    ///
    /// If this information has already been provided prior.
    pub fn update_end_of_then_reachability(&mut self, reachable: bool) {
        assert!(self.end_of_then_is_reachable.is_none());
        self.end_of_then_is_reachable = Some(reachable);
    }

    /// Returns `true` if the end of the `then` branch is reachable.
    ///
    /// Returns `None` if `else` was never visited.
    #[track_caller]
    pub fn is_end_of_then_reachable(&self) -> Option<bool> {
        self.end_of_then_is_reachable
    }

    /// Informs the [`IfControlFrame`] that the `else` block has been visited.
    pub fn visited_else(&mut self) {
        self.visited_else = true;
    }

    /// Returns `true` if the `else` block has been visited.
    pub fn has_visited_else(&self) -> bool {
        self.visited_else
    }

    /// Returns the [`BlockHeight`] of the [`IfControlFrame`].
    pub fn block_height(&self) -> BlockHeight {
        self.stack_height
    }

    /// Returns the [`BlockType`] of the [`IfControlFrame`].
    pub fn block_type(&self) -> BlockType {
        self.block_type
    }

    /// Returns a reference to the [`ConsumeFuel`] instruction of the [`IfControlFrame`] if any.
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
    /// # Panics
    ///
    /// If the `consume_fuel` field was not already `Some`.
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
    pub fn new(kind: ControlFrameKind) -> Self {
        Self { kind }
    }

    /// Returns the [`ControlFrameKind`] of the [`UnreachableControlFrame`].
    pub fn kind(&self) -> ControlFrameKind {
        self.kind
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
    /// Returns an iterator over the registers holding the branch parameters of the [`ControlFrame`].
    pub fn branch_params(&self, engine: &Engine) -> BoundedRegSpan {
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

    /// Makes the [`ControlFrame`] aware that there is a branch to it.
    pub fn branch_to(&mut self) {
        match self {
            ControlFrame::Block(frame) => frame.branch_to(),
            ControlFrame::Loop(frame) => frame.branch_to(),
            ControlFrame::If(frame) => frame.branch_to(),
            Self::Unreachable(frame) => {
                panic!("tried to `bump_branches` on an unreachable control frame: {frame:?}")
            }
        }
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
