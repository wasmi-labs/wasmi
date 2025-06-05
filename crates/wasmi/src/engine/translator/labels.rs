use crate::{engine::translator::Instr, ir::BranchOffset, Error};
use alloc::vec::Vec;
use core::{
    fmt::{self, Display},
    slice::Iter as SliceIter,
};

/// A label during the Wasmi compilation process.
#[derive(Debug, Copy, Clone)]
pub enum Label {
    /// The label has already been pinned to a particular [`Instr`].
    Pinned(Instr),
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
    user: Instr,
}

impl LabelUser {
    /// Creates a new [`LabelUser`].
    pub fn new(label: LabelRef, user: Instr) -> Self {
        Self { label, user }
    }
}

/// An error that may occur while operating on the [`LabelRegistry`].
#[derive(Debug, Copy, Clone)]
pub enum LabelError {
    /// When trying to pin an already pinned [`Label`].
    AlreadyPinned { label: LabelRef, pinned_to: Instr },
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
    /// If the `label` has already been pinned to some other [`Instr`].
    pub fn pin_label(&mut self, label: LabelRef, instr: Instr) -> Result<(), LabelError> {
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
    pub fn try_pin_label(&mut self, label: LabelRef, instr: Instr) {
        if let unpinned @ Label::Unpinned = self.get_label_mut(label) {
            *unpinned = Label::Pinned(instr)
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
        user: Instr,
    ) -> Result<BranchOffset, Error> {
        let offset = match *self.get_label(label) {
            Label::Pinned(target) => {
                BranchOffset::from_src_to_dst(user.into_u32(), target.into_u32())?
            }
            Label::Unpinned => {
                self.users.push(LabelUser::new(label, user));
                BranchOffset::uninit()
            }
        };
        Ok(offset)
    }

    /// Resolves a `label` to its pinned [`Instr`].
    ///
    /// # Errors
    ///
    /// If the `label` is unpinned.
    fn resolve_label(&self, label: LabelRef) -> Result<Instr, LabelError> {
        match self.get_label(label) {
            Label::Pinned(instr) => Ok(*instr),
            Label::Unpinned => Err(LabelError::Unpinned { label }),
        }
    }

    /// Returns an iterator over pairs of user [`Instr`] and their [`BranchOffset`].
    ///
    /// # Panics
    ///
    /// If used before all used branching labels have been pinned.
    pub fn resolved_users(&self) -> ResolvedUserIter {
        ResolvedUserIter {
            users: self.users.iter(),
            registry: self,
        }
    }
}

/// Iterator over resolved label users.
///
/// Iterates over pairs of user [`Instr`] and its respective [`BranchOffset`]
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
    type Item = (Instr, Result<BranchOffset, Error>);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.users.next()?;
        let src = next.user;
        let dst = self
            .registry
            .resolve_label(next.label)
            .unwrap_or_else(|err| panic!("failed to resolve user: {err}"));
        let offset =
            BranchOffset::from_src_to_dst(src.into_u32(), dst.into_u32()).map_err(Into::into);
        Some((src, offset))
    }
}
