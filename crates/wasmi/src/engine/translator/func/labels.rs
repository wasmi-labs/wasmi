use crate::{
    engine::{
        translator::func::{
            encoder::{BytePos, Pos},
            utils::Reset,
        },
        TranslationError,
    },
    ir::{BranchOffset, Op},
    Error,
};
use alloc::vec::Vec;
use core::{
    error::Error as CoreError,
    fmt::{self, Display},
    slice::Iter as SliceIter,
};

/// A label during the Wasmi compilation process.
#[derive(Debug, Copy, Clone)]
pub enum Label {
    /// The label has already been pinned to a particular [`Pos<Op>`].
    Pinned(Pos<Op>),
    /// The label is still unpinned.
    Unpinned,
}

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
    fn get(&self, lref: LabelRef) -> &Label {
        &self.labels[usize::from(lref)]
    }

    /// Returns an exclusive reference to the underlying [`Label`].
    #[inline]
    fn get_mut(&mut self, lref: LabelRef) -> &mut Label {
        &mut self.labels[usize::from(lref)]
    }
}

/// A user of a label.
#[derive(Debug, Copy, Clone)]
pub struct LabelUser {
    /// The branch `target` label in use by the `user`.
    target: LabelRef,
    /// The reference to the using instruction.
    user: Pos<Op>,
    /// The [`BranchOffset`] of `user` that needs to be updated once `target` has been resolved.
    offset: Pos<BranchOffset>,
}

impl LabelUser {
    /// Creates a new [`LabelUser`].
    pub fn new(target: LabelRef, user: Pos<Op>, offset: Pos<BranchOffset>) -> Self {
        Self {
            target,
            user,
            offset,
        }
    }
}

/// An error that may occur while operating on the [`LabelRegistry`].
#[derive(Debug, Copy, Clone)]
pub enum LabelError {
    /// When trying to pin an already pinned [`Label`].
    AlreadyPinned { label: LabelRef, target: Pos<Op> },
    /// When trying to resolve an unpinned [`Label`].
    Unpinned { label: LabelRef },
}

impl LabelError {
    /// Creates a [`LabelError::AlreadyPinned`] with `label` and `target`.
    #[cold]
    #[inline]
    fn already_pinned(label: LabelRef, target: Pos<Op>) -> Self {
        Self::AlreadyPinned { label, target }
    }

    /// Creates a [`LabelError::Unpinned`] with `label`.
    #[cold]
    #[inline]
    fn unpinned(label: LabelRef) -> Self {
        Self::Unpinned { label }
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
            Self::Unpinned { label } => {
                write!(f, "trying to resolve unpinned label: {label:?}")
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

    /// Pins the `label` to the given `target` [`Pos<Op>`].
    ///
    /// # Errors
    ///
    /// If the `label` has already been pinned to some other [`Pos<Op>`].
    pub fn pin_label(&mut self, label: LabelRef, target: Pos<Op>) -> Result<(), LabelError> {
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

    /// Tries to resolve the `label`.
    ///
    /// Returns the proper `BranchOffset` in case the `label` has already been
    /// pinned and returns an uninitialized `BranchOffset` otherwise.
    ///
    /// In case the `label` has not yet been pinned the `user` is registered
    /// for deferred label resolution.
    pub fn try_resolve_label(
        &mut self,
        label: LabelRef,
        user: Pos<Op>,
    ) -> Result<BranchOffset, Error> {
        let offset = match *self.labels.get(label) {
            Label::Pinned(target) => trace_branch_offset(user, target)?,
            Label::Unpinned => {
                self.users.push(LabelUser::new(label, user));
                BranchOffset::uninit()
            }
        };
        Ok(offset)
    }

    /// Resolves a `label` to its pinned [`OpPos`].
    ///
    /// # Errors
    ///
    /// If the `label` is unpinned.
    fn resolve_label(&self, label: LabelRef) -> Result<Pos<Op>, LabelError> {
        match self.labels.get(label) {
            Label::Pinned(target) => Ok(*target),
            Label::Unpinned => Err(LabelError::unpinned(label)),
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
            registry: self,
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
    registry: &'a LabelRegistry,
}

impl Iterator for ResolvedUserIter<'_> {
    type Item = (Pos<Op>, Result<BranchOffset, Error>);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.users.next()?;
        let src = next.user;
        let dst = self
            .registry
            .resolve_label(next.target)
            .unwrap_or_else(|err| panic!("failed to resolve user: {err}"));
        let offset = trace_branch_offset(src, dst);
        Some((src, offset))
    }
}

/// Creates an initialized [`BranchOffset`] from `src` to `dst`.
///
/// # Errors
///
/// If the resulting [`BranchOffset`] is out of bounds.
fn trace_branch_offset(src: Pos<Op>, dst: Pos<Op>) -> Result<BranchOffset, Error> {
    fn trace_offset_or_none(src: Pos<Op>, dst: Pos<Op>) -> Option<BranchOffset> {
        let src = isize::try_from(usize::from(BytePos::from(src))).ok()?;
        let dst = isize::try_from(usize::from(BytePos::from(dst))).ok()?;
        let offset = dst.checked_sub(src)?;
        i32::try_from(offset).map(BranchOffset::from).ok()
    }
    let Some(offset) = trace_offset_or_none(src, dst) else {
        return Err(Error::from(TranslationError::BranchOffsetOutOfBounds));
    };
    Ok(offset)
}
