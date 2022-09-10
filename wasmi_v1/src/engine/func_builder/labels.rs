use super::Instr;
use alloc::vec::Vec;
use core::fmt::{self, Display};

/// A label during the `wasmi` compilation process.
#[derive(Debug, Copy, Clone)]
pub enum Label {
    /// The label has already been pinned to a particular [`Instr`].
    Pinned(Instr),
    /// The label is still unpinned.
    Unpinned,
}

/// A reference to an [`Label`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct LabelRef(usize);

/// The label registry.
///
/// Allows to allocate new labels pin them and resolve pinned ones.
#[derive(Debug, Default)]
pub struct LabelRegistry {
    labels: Vec<Label>,
}

/// An error that may occur while operating on the [`LabelRegistry`].
#[derive(Debug, Copy, Clone)]
pub enum LabelError {
    /// When trying to pin an already pinned [`Label`].
    AlreadyPinned { label: LabelRef, pinned_to: Instr },
    /// When trying to resolve an unpinned [`Label`].
    Unpinned { label: LabelRef },
}

impl Display for LabelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LabelError::AlreadyPinned { label, pinned_to } => {
                write!(
                    f,
                    "trying to pin already pinned label {label:?} (pinned to {pinned_to:?})"
                )
            }
            LabelError::Unpinned { label } => {
                write!(f, "trying to resolve unpinned label: {label:?}")
            }
        }
    }
}

impl LabelRegistry {
    /// Resets the [`LabelRegistry`] for reuse.
    pub fn reset(&mut self) {
        self.labels.clear()
    }

    /// Allocates a new unpinned [`Label`].
    pub fn new_label(&mut self) -> LabelRef {
        let index = self.labels.len();
        self.labels.push(Label::Unpinned);
        LabelRef(index)
    }

    /// Pins the `label` to the given `instr`.
    ///
    /// # Errors
    ///
    /// If the `label` has already been pinned to some other [`Instr`].
    pub fn pin_label(&mut self, label: LabelRef, instr: Instr) -> Result<(), LabelError> {
        match &mut self.labels[label.0] {
            label @ Label::Unpinned => {
                *label = Label::Pinned(instr);
            }
            Label::Pinned(pinned) => {
                return Err(LabelError::AlreadyPinned {
                    label,
                    pinned_to: *pinned,
                })
            }
        }
        Ok(())
    }

    /// Pins the `label` to the given `instr` if unpinned.
    pub fn try_pin_label(&mut self, label: LabelRef, instr: Instr) {
        let label = &mut self.labels[label.0];
        if matches!(label, Label::Unpinned) {
            *label = Label::Pinned(instr)
        }
    }

    /// Resolves a `label` to its pinned [`Instr`].
    ///
    /// # Errors
    ///
    /// If the `label` is unpinned.
    pub fn resolve_label(&self, label: LabelRef) -> Result<Instr, LabelError> {
        match &self.labels[label.0] {
            Label::Pinned(instr) => Ok(*instr),
            Label::Unpinned => Err(LabelError::Unpinned { label }),
        }
    }
}
