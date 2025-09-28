use crate::{
    engine::{
        translator::func::encoder::{BytePos, Pos},
        translator::func::utils::Reset,
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
    /// All registered labels, pinned or unpinned.
    labels: Vec<Label>,
    /// All label users that could not be immediately resolved.
    users: Vec<LabelUser>,
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
    AlreadyPinned { label: LabelRef, pinned_to: Pos<Op> },
    /// When trying to resolve an unpinned [`Label`].
    Unpinned { label: LabelRef },
}

impl CoreError for LabelError {}

impl Display for LabelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyPinned { label, pinned_to } => {
                write!(
                    f,
                    "trying to pin already pinned label {label:?} (pinned to {pinned_to:?})"
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
        self.labels.clear();
        self.users.clear();
    }
}

impl LabelRegistry {
    /// Allocates a new unpinned [`Label`].
    pub fn new_label(&mut self) -> LabelRef {
        let index = self.labels.len();
        self.labels.push(Label::Unpinned);
        LabelRef::from(index)
    }

    /// Returns a shared reference to the underlying [`Label`].
    #[inline]
    fn get_label(&self, label: LabelRef) -> &Label {
        &self.labels[usize::from(label)]
    }

    /// Returns an exclusive reference to the underlying [`Label`].
    #[inline]
    fn get_label_mut(&mut self, label: LabelRef) -> &mut Label {
        &mut self.labels[usize::from(label)]
    }

    /// Pins the `label` to the given `instr`.
    ///
    /// # Errors
    ///
    /// If the `label` has already been pinned to some other [`OpPos`].
    pub fn pin_label(&mut self, label: LabelRef, instr: Pos<Op>) -> Result<(), LabelError> {
        match self.get_label_mut(label) {
            Label::Pinned(pinned) => Err(LabelError::AlreadyPinned {
                label,
                pinned_to: *pinned,
            }),
            unpinned @ Label::Unpinned => {
                *unpinned = Label::Pinned(instr);
                Ok(())
            }
        }
    }

    /// Pins the `label` to the given `instr` if unpinned.
    pub fn try_pin_label(&mut self, label: LabelRef, instr: Pos<Op>) {
        if let unpinned @ Label::Unpinned = self.get_label_mut(label) {
            *unpinned = Label::Pinned(instr)
        }
    }

    /// Creates an initialized [`BranchOffset`] from `src` to `dst`.
    ///
    /// # Errors
    ///
    /// If the resulting [`BranchOffset`] is out of bounds.
    pub fn trace_branch_offset(src: Pos<Op>, dst: Pos<Op>) -> Result<BranchOffset, Error> {
        fn trace_offset32(src: Pos<Op>, dst: Pos<Op>) -> Option<i32> {
            let src = isize::try_from(usize::from(BytePos::from(src))).ok()?;
            let dst = isize::try_from(usize::from(BytePos::from(dst))).ok()?;
            let offset = dst.checked_sub(src)?;
            i32::try_from(offset).ok()
        }
        let Some(offset) = trace_offset32(src, dst) else {
            return Err(Error::from(TranslationError::BranchOffsetOutOfBounds));
        };
        Ok(BranchOffset::from(offset))
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
        let offset = match *self.get_label(label) {
            Label::Pinned(target) => Self::trace_branch_offset(user, target)?,
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
        match self.get_label(label) {
            Label::Pinned(instr) => Ok(*instr),
            Label::Unpinned => Err(LabelError::Unpinned { label }),
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
        let offset = LabelRegistry::trace_branch_offset(src, dst);
        Some((src, offset))
    }
}
