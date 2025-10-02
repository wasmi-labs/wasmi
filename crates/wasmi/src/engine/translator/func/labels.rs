use crate::{
    engine::translator::func::{
        encoder::{BytePos, Pos},
        utils::Reset,
    },
    ir::{BranchOffset, Op},
};
use alloc::vec::Vec;
use core::{
    error::Error as CoreError,
    fmt::{self, Display},
    slice::Iter as SliceIter,
};

/// A reference to an [`Label`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct LabelRef(usize);

impl From<usize> for LabelRef {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<LabelRef> for usize {
    #[inline]
    fn from(value: LabelRef) -> Self {
        value.0
    }
}

/// The label registry.
///
/// Allows to allocate new labels pin them and resolve pinned ones.
#[derive(Debug, Default)]
pub struct LabelRegistry {
    /// Registered labels and their states: pinned or unpinned.
    labels: RegisteredLabels,
    /// Label users that could not be immediately resolved.
    users: Vec<LabelUser>,
}

/// All registered labels.
#[derive(Debug, Default)]
pub struct RegisteredLabels {
    labels: Vec<Label>,
}

/// A label during the Wasmi compilation process.
#[derive(Debug, Copy, Clone)]
pub enum Label {
    /// The label has already been pinned to a particular [`Pos<Op>`].
    Pinned(Pos<Op>),
    /// The label is still unpinned.
    Unpinned,
}

impl Reset for RegisteredLabels {
    fn reset(&mut self) {
        self.labels.clear();
    }
}

impl RegisteredLabels {
    /// Pushes a new [`Label::Unpinned`] to `self` and returns its [`LabelRef`].
    #[inline]
    pub fn push(&mut self) -> LabelRef {
        let index = self.labels.len();
        self.labels.push(Label::Unpinned);
        LabelRef::from(index)
    }

    /// Returns a shared reference to the underlying [`Label`].
    #[inline]
    fn get(&self, lref: LabelRef) -> Label {
        self.labels[usize::from(lref)]
    }

    /// Returns an exclusive reference to the underlying [`Label`].
    #[inline]
    fn get_mut(&mut self, lref: LabelRef) -> &mut Label {
        &mut self.labels[usize::from(lref)]
    }
}

/// A user of a label.
#[derive(Debug, Copy, Clone)]
struct LabelUser {
    /// The branch destination [`Label`].
    dst: LabelRef,
    /// The generic branch source.
    src: BytePos,
    /// The [`BranchOffset`] of `src` that needs to be updated once `dst` has been pinned.
    pos: Pos<BranchOffset>,
}

/// An error that may occur while operating on the [`LabelRegistry`].
#[derive(Debug, Copy, Clone)]
enum LabelError {
    /// When trying to pin an already pinned [`Label`].
    AlreadyPinned { label: LabelRef, target: Pos<Op> },
}

impl LabelError {
    /// Creates a [`LabelError::AlreadyPinned`] with `label` and `target`.
    #[cold]
    #[inline]
    fn already_pinned(label: LabelRef, target: Pos<Op>) -> Self {
        Self::AlreadyPinned { label, target }
    }
}

impl CoreError for LabelError {}
impl Display for LabelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyPinned { label, target } => {
                write!(
                    f,
                    "trying to pin already pinned label {label:?} (pinned to {target:?})"
                )
            }
        }
    }
}

impl Reset for LabelRegistry {
    fn reset(&mut self) {
        self.labels.reset();
        self.users.clear();
    }
}

impl LabelRegistry {
    /// Allocates a new unpinned [`Label`].
    pub fn new_label(&mut self) -> LabelRef {
        self.labels.push()
    }

    /// Pushes a new label user to the [`LabelRegistry`] for deferred resolution.
    pub fn new_user(&mut self, dst: LabelRef, src: BytePos, pos: Pos<BranchOffset>) {
        self.users.push(LabelUser { dst, src, pos })
    }

    /// Returns the [`Label`] at `lref`.
    pub fn get_label(&self, lref: LabelRef) -> Label {
        self.labels.get(lref)
    }

    /// Returns `true` if [`Label`] at `lref` is pinned.
    pub fn is_pinned(&self, lref: LabelRef) -> bool {
        matches!(self.get_label(lref), Label::Pinned(_))
    }

    /// Pins the `label` to the given `target` [`Pos<Op>`].
    ///
    /// # Errors
    ///
    /// If the `label` has already been pinned to some other [`Pos<Op>`].
    #[track_caller]
    pub fn pin_label(&mut self, label: LabelRef, target: Pos<Op>) {
        if let Err(error) = self.pin_label_or_err(label, target) {
            panic!("failed to pin label: {error}")
        }
    }

    /// Pins the `label` to the given `target` [`Pos<Op>`].
    ///
    /// # Errors
    ///
    /// If the `label` has already been pinned to some other [`Pos<Op>`].
    fn pin_label_or_err(&mut self, label: LabelRef, target: Pos<Op>) -> Result<(), LabelError> {
        let cell = self.labels.get_mut(label);
        if let Label::Pinned(pinned) = cell {
            return Err(LabelError::already_pinned(label, *pinned));
        }
        debug_assert!(matches!(cell, Label::Unpinned));
        *cell = Label::Pinned(target);
        Ok(())
    }

    /// Pins the `label` to the given `target` if unpinned.
    ///
    /// # Note
    ///
    /// Does nothing if the `label` is already pinned.
    pub fn pin_label_if_unpinned(&mut self, label: LabelRef, target: Pos<Op>) {
        let cell = self.labels.get_mut(label);
        if matches!(cell, Label::Unpinned) {
            *cell = Label::Pinned(target)
        }
    }

    /// Returns an iterator over pairs of user [`OpPos`] and their [`BranchOffset`].
    ///
    /// # Panics
    ///
    /// If used before all used branching labels have been pinned.
    pub fn resolved_users(&self) -> ResolvedUserIter<'_> {
        ResolvedUserIter {
            users: self.users.iter(),
            labels: &self.labels,
        }
    }
}

/// Iterator over resolved label users.
///
/// Iterates over pairs of user [`OpPos`] and its respective [`BranchOffset`]
/// which allows the [`InstructionsBuilder`] to properly update the branching
/// offsets.
///
/// [`InstructionsBuilder`]: [`super::InstructionsBuilder`]
#[derive(Debug)]
pub struct ResolvedUserIter<'a> {
    users: SliceIter<'a, LabelUser>,
    labels: &'a RegisteredLabels,
}

/// A fully resolved [`Label`] user.
#[derive(Debug, Copy, Clone)]
pub struct ResolvedLabelUser {
    pub src: BytePos,
    pub dst: Pos<Op>,
    pub pos: Pos<BranchOffset>,
}

impl Iterator for ResolvedUserIter<'_> {
    type Item = ResolvedLabelUser;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.users.next()?;
        let src = next.src;
        let pos = next.pos;
        let Label::Pinned(dst) = self.labels.get(next.dst) else {
            panic!("encountered unexpected unpinned label: {:?}", next.dst)
        };
        Some(ResolvedLabelUser { src, dst, pos })
    }
}
