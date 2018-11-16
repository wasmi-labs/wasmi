mod ol {
	#[allow(unused_imports)]
	use alloc::prelude::*;

	use core::fmt;
	#[cfg(feature = "std")]
	use std::error;

	#[derive(Debug)]
	pub struct Error(String);

	impl fmt::Display for Error {
		fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
			write!(f, "{}", self.0)
		}
	}

	#[cfg(feature = "std")]
	impl error::Error for Error {
		fn description(&self) -> &str {
			&self.0
		}
	}

	/// Stack with limit.
	#[derive(Debug)]
	pub struct StackWithLimit<T>
	where
		T: Clone,
	{
		/// Stack values.
		values: Vec<T>,
		/// Stack limit (maximal stack len).
		limit: usize,
	}

	impl<T> StackWithLimit<T>
	where
		T: Clone,
	{
		pub fn with_limit(limit: usize) -> Self {
			StackWithLimit {
				values: Vec::new(),
				limit: limit,
			}
		}

		pub fn is_empty(&self) -> bool {
			self.values.is_empty()
		}

		pub fn len(&self) -> usize {
			self.values.len()
		}

		pub fn top(&self) -> Result<&T, Error> {
			self.values
				.last()
				.ok_or_else(|| Error("non-empty stack expected".into()))
		}

		pub fn top_mut(&mut self) -> Result<&mut T, Error> {
			self.values
				.last_mut()
				.ok_or_else(|| Error("non-empty stack expected".into()))
		}

		// Not the same as vector.get
		pub fn get(&self, index: usize) -> Result<&T, Error> {
			if index >= self.values.len() {
				return Err(Error(format!(
					"trying to get value at position {} on stack of size {}",
					index,
					self.values.len()
				)));
			}

			Ok(self
				.values
				.get(self.values.len() - 1 - index)
				.expect("checked couple of lines above"))
		}

		pub fn push(&mut self, value: T) -> Result<(), Error> {
			if self.values.len() >= self.limit {
				return Err(Error(format!("exceeded stack limit {}", self.limit)));
			}

			self.values.push(value);
			Ok(())
		}

		pub fn pop(&mut self) -> Result<T, Error> {
			self.values
				.pop()
				.ok_or_else(|| Error("non-empty stack expected".into()))
		}

		pub fn resize(&mut self, new_size: usize, dummy: T) {
			debug_assert!(new_size <= self.values.len());
			self.values.resize(new_size, dummy);
		}
	}
}

#[derive(Copy, Clone, Debug)]
pub struct StackOverflow;

// impl From<StackOverflow> for TrapKind {
// 	fn from(_: StackOverflow) -> TrapKind {
// 		TrapKind::StackOverflow
// 	}
// }

// impl From<StackOverflow> for Trap {
// 	fn from(_: StackOverflow) -> Trap {
// 		Trap::new(TrapKind::StackOverflow)
// 	}
// }

// TODO: impl constructors
struct Limit(usize);
struct InitialSize(usize);

/// Pre-allocated, growable stack with upper bound on size.
///
/// StackWithLimit is guaranteed never to grow larger than a set limit.
/// When limit is reached attempting to push to the stack will return
/// `Err(StackOverflow)`.
///
/// In addition to the limit. Initial stack size is configurable.
/// `StackWithLimit` will start out with initial size, but grow if necessary.
#[derive(Debug)]
pub struct StackWithLimit<T> {
	stack: Vec<T>,
	limit: usize,
}

impl<T> StackWithLimit<T> {
	/// Create an new StackWithLimit with `limit` max size and `initial_size` elements pre-allocated
	/// `initial_size` should not be larger than `limit`
	pub fn new(initial_size: usize, limit: usize) -> StackWithLimit<T> {
		debug_assert!(
			limit >= initial_size,
			"initial_size should not be larger than StackWithLimit limit"
		);
		use std::cmp::min;
		StackWithLimit {
			stack: Vec::with_capacity(initial_size),
			limit: min(initial_size, limit),
		}
	}

	/// Create an new StackWithLimit with `limit` max size and `limit` elements pre-allocated
	pub fn with_limit(limit: usize) -> StackWithLimit<T> {
		StackWithLimit {
			stack: Vec::with_capacity(limit),
			limit: limit,
		}
	}

	/// Attempt to push value onto stack.
	///
	/// # Errors
	///
	/// Returns Err(StackOverflow) if stack is already full.
	pub(crate) fn push(&mut self, value: T) -> Result<(), StackOverflow> {
		debug_assert!(
			self.stack.len() <= self.limit,
			"Stack length should never be larger than stack limit."
		);
		if self.stack.len() < self.limit {
			self.stack.push(value);
			Ok(())
		} else {
			Err(StackOverflow)
		}
	}

	pub(crate) fn pop(&mut self) -> Option<T> {
		debug_assert!(
			self.stack.len() <= self.limit,
			"Stack length should never be larger than stack limit."
		);
		self.stack.pop()
	}

	/// Return optional reference to item in stack
	///
	/// `bstack.get_relative_to_top(0)` gets the top of the stack
	///
	/// `bstack.get_relative_to_top(1)` gets the item just below the stack
	///

	pub(crate) fn get_relative_to_top(&self, depth: usize) -> Option<&T> {
		let offset = depth + 1;
		if self.stack.len() < offset {
			None
		} else {
			// We should be cognizant of integer underflow here.
			// If offset > len(), (len() - offset) will underflow.
			// In debug builds, underflow panics, but in release mode, underflow is not checked.
			self.stack.get(self.stack.len() - offset)
		}
	}

	pub(crate) fn top(&self) -> Option<&T> {
		self.stack.last()
	}

	pub(crate) fn top_mut(&mut self) -> Option<&mut T> {
		self.stack.last_mut()
	}

	pub(crate) fn truncate(&mut self, new_size: usize) {
		self.stack.truncate(new_size)
	}

	pub(crate) fn len(&self) -> usize {
		self.stack.len()
	}

	pub(crate) fn is_empty(&self) -> bool {
		self.stack.is_empty()
	}

	// /// return a new empty StackWithLimit with limit equal to the amount of room
	// /// this stack has available
	// pub fn spare(&self) -> StackWithLimit<T> {
	//     // This will be used to allocate new stacks when calling into other wasm modules
	//     StackWithLimit::new(0, self.limit - self.len())
	// }
}

#[cfg(test)]
mod test {
	use super::StackWithLimit;

	fn get_relative_to_top() {
		let mut bstack = StackWithLimit::<i32>::with_limit(2);
		bstack.push(1).unwrap();
		bstack.push(2).unwrap();
		bstack.push(3).unwrap_err();
		assert_eq!(bstack.get_relative_to_top(0), Some(&2));
		assert_eq!(bstack.get_relative_to_top(1), Some(&1));
		assert_eq!(bstack.get_relative_to_top(2), None);
	}
}
