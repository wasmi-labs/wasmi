use crate::{
    engine::{translator::func::encoder::BytePos, TranslationError},
    ir::BranchOffset,
    Error,
};
use alloc::vec::Vec;
use core::{
    fmt::{self, Display},
    slice::Iter as SliceIter,
};

/// A label during the Wasmi compilation process.
#[derive(Debug, Copy, Clone)]
pub enum Label {
    /// The label has already been pinned to a particular [`OpPos`].
    Pinned(BytePos),
    /// The label is still unpinned.
    Unpinned,
}

/// A reference to an [`Label`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct LabelRef(u32);

impl LabelRef {
    /// Returns the `usize` value of the [`LabelRef`].
    #[inline]
    fn into_usize(self) -> usize {
        self.0 as usize
    }
}

/// The label registry.
///
/// Allows to allocate new labels pin them and resolve pinned ones.
#[derive(Debug, Default)]
pub struct LabelRegistry {
    labels: Vec<Label>,
    users: Vec<LabelUser>,
}

/// A user of a label.
#[derive(Debug)]
pub struct LabelUser {
    /// The label in use by the user.
    label: LabelRef,
    /// The reference to the using instruction.
    user: BytePos,
}

impl LabelUser {
    /// Creates a new [`LabelUser`].
    pub fn new(label: LabelRef, user: BytePos) -> Self {
        Self { label, user }
    }
}

/// An error that may occur while operating on the [`LabelRegistry`].
#[derive(Debug, Copy, Clone)]
pub enum LabelError {
    /// When trying to pin an already pinned [`Label`].
    AlreadyPinned { label: LabelRef, pinned_to: BytePos },
    /// When trying to resolve an unpinned [`Label`].
    Unpinned { label: LabelRef },
}

impl core::error::Error for LabelError {}

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
        self.labels.clear();
        self.users.clear();
    }

    /// Allocates a new unpinned [`Label`].
    pub fn new_label(&mut self) -> LabelRef {
        let index: u32 = self
            .labels
            .len()
            .try_into()
            .unwrap_or_else(|err| panic!("cannot have more than u32::MAX label refs: {err}"));
        self.labels.push(Label::Unpinned);
        LabelRef(index)
    }

    /// Returns a shared reference to the underlying [`Label`].
    #[inline]
    fn get_label(&self, label: LabelRef) -> &Label {
        &self.labels[label.into_usize()]
    }

    /// Returns an exclusive reference to the underlying [`Label`].
    #[inline]
    fn get_label_mut(&mut self, label: LabelRef) -> &mut Label {
        &mut self.labels[label.into_usize()]
    }

    /// Pins the `label` to the given `instr`.
    ///
    /// # Errors
    ///
    /// If the `label` has already been pinned to some other [`OpPos`].
    pub fn pin_label(&mut self, label: LabelRef, instr: BytePos) -> Result<(), LabelError> {
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
    pub fn try_pin_label(&mut self, label: LabelRef, instr: BytePos) {
        if let unpinned @ Label::Unpinned = self.get_label_mut(label) {
            *unpinned = Label::Pinned(instr)
        }
    }

    /// Creates an initialized [`BranchOffset`] from `src` to `dst`.
    ///
    /// # Errors
    ///
    /// If the resulting [`BranchOffset`] is out of bounds.
    pub fn trace_branch_offset(src: BytePos, dst: BytePos) -> Result<BranchOffset, Error> {
        fn trace_offset32(src: BytePos, dst: BytePos) -> Option<i32> {
            let src = isize::try_from(usize::from(src)).ok()?;
            let dst = isize::try_from(usize::from(dst)).ok()?;
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
        user: BytePos,
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
    fn resolve_label(&self, label: LabelRef) -> Result<BytePos, LabelError> {
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
    type Item = (BytePos, Result<BranchOffset, Error>);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.users.next()?;
        let src = next.user;
        let dst = self
            .registry
            .resolve_label(next.label)
            .unwrap_or_else(|err| panic!("failed to resolve user: {err}"));
        let offset = LabelRegistry::trace_branch_offset(src, dst);
        Some((src, offset))
    }
}
