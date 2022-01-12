use super::super::super::LabelIdx;

/// A control flow frame.
#[derive(Debug)]
pub enum ControlFrame {
    /// Basic block control frame.
    ///
    /// Branches within this frame jump to the end of the [`ControlFrame::Block`].
    Block {
        /// Label representing the end of the [`ControlFrame::Block`].
        end_label: LabelIdx,
    },
    /// Loop control frame.
    ///
    /// Branches within this frame jump to the head of the [`ControlFrame::Loop`].
    Loop {
        /// Label representing the [`ControlFrame::Loop`] header.
        header: LabelIdx,
    },
    /// The then-block of an `If` control frame.
    ///
    /// Branches within this frame jump to the end of the [`ControlFrame::If`].
    If {
        /// Label representing the end of the [`ControlFrame::If`].
        end_label: LabelIdx,
        /// If the condition of the `if` statement that this [`ControlFrame`]
        /// represents evaluates to `false` control flow jumps to this label.
        /// Note that during compilation of the Wasm instructions it is not
        /// known upfront whether a seen `If` control frame actually has an
        /// accompanied `Else` [`ControlFrame`]. Therefore this is required to
        /// be kept in sync once the `If` is finished or the `Else` is seen.
        ///
        /// As long as no `Else` control frame is seen this label index is equal
        /// to the `end_label`.
        else_label: LabelIdx,
    },
    /// The optional else-block of an `If` control frame.
    Else {
        /// Label representing the end of the associated [`ControlFrame::If`].
        end_label: LabelIdx,
    },
}

impl ControlFrame {
    /// Returns the label for the branch destination of the [`ControlFrame`].
    pub fn br_destination(&self) -> LabelIdx {
        match self {
            ControlFrame::Block { end_label } => *end_label,
            ControlFrame::Loop { header } => *header,
            ControlFrame::If { end_label, .. } => *end_label,
            ControlFrame::Else { end_label } => *end_label,
        }
    }

    /// Returns a label which should be resolved at the `End` Wasm opcode.
    ///
    /// All [`ControlFrame`] kinds have it except [`ControlFrame::Loop`].
    /// In order to a [`ControlFrame::Loop`] to branch outside it is required
    /// to be wrapped in another control frame such as [`ControlFrame::Block`].
    pub fn end_label(&self) -> LabelIdx {
        match *self {
            ControlFrame::Block { end_label } => end_label,
            ControlFrame::If { end_label, .. } => end_label,
            ControlFrame::Else { end_label } => end_label,
            ControlFrame::Loop { .. } => panic!(
                "tried to receive `end_label` which is not supported for loop control frames: {:?}",
                self
            ),
        }
    }
}
