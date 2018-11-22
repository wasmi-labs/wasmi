use core::marker::PhantomData;
use core::ptr;
use core::usize;

#[allow(unused_imports)]
use alloc::prelude::*;

#[derive(Copy, Clone, Debug)]
pub struct StackOverflow;

/// Pre-allocated, growable stack with upper bound on size.
///
/// StackWithLimit is guaranteed never to grow larger than a set limit.
/// When limit is reached attempting to push to the stack will return
/// `Err(StackOverflow)`.
///
/// Both limit and initial stack size are configurable.
/// `StackWithLimit` will start out with initial size, but grow when necessary.
#[derive(Debug)]
pub struct StackWithLimit<T> {
	stack: Vec<T>,
	limit: usize,
}

impl<T> StackWithLimit<T> {
	/// Create a StackWithLimit with `limit` max size and `initial_size` of pre-allocated
	/// memory.
	///
	/// ```
	/// # extern crate wasmi;
	/// # use wasmi::{StackWithLimit, StackSize, FuncRef};
	/// StackWithLimit::<FuncRef>::new(
	/// 	StackSize::from_element_count(1024).into_initial(),
	/// 	StackSize::from_element_count(2048).into_limit(),
	/// );
	/// ```
	///
	/// Unlimited
	///
	/// ```
	/// # extern crate wasmi;
	/// # use wasmi::{StackWithLimit, StackSize, RuntimeValue};
	/// StackWithLimit::<RuntimeValue>::new(
	/// 	StackSize::from_element_count(1024).into_initial(),
	/// 	StackSize::unlimited(),
	/// );
	/// ```
	///
	/// # Panics
	///
	/// In debug mode, panics if `initial_size` is larger than `limit`.
	pub fn new(initial_size: StackSizeInitial<T>, limit: StackSizeLimit<T>) -> StackWithLimit<T> {
		let initial_size_elements = initial_size.0.element_count();
		let limit_elements = limit.0.element_count();
		debug_assert!(
			limit_elements >= initial_size_elements,
			"initial_size should not be larger than StackWithLimit limit"
		);
		StackWithLimit {
			stack: Vec::with_capacity(initial_size_elements.min(limit_elements)),
			limit: limit_elements,
		}
	}

	/// Create an new StackWithLimit with `limit` max size and `limit` elements pre-allocated
	///
	/// ```
	/// # extern crate wasmi;
	/// # use wasmi::{StackWithLimit, StackSize};
	/// let bstack = StackWithLimit::<i32>::with_size(StackSize::from_element_count(2));
	/// ```
	pub fn with_size(size: StackSize<T>) -> StackWithLimit<T> {
		StackWithLimit::new(StackSizeInitial(size), StackSizeLimit(size))
	}

	/// Attempt to push value onto stack.
	///
	/// # Errors
	///
	/// Returns Err(StackOverflow) if stack is already full.
	#[inline]
	pub fn push(&mut self, value: T) -> Result<(), StackOverflow> {
		if self.stack.len() == self.stack.capacity() {
			if self.stack.len() == self.limit {
				return Err(StackOverflow);
			}
			// grows exponentially, just like Vec normally does
			// https://doc.rust-lang.org/1.26.0/src/alloc/raw_vec.rs.html#462
			let desired_len = self
				.stack
				.len()
				.checked_mul(2)
				.unwrap_or(usize::MAX)
				.min(self.limit)
				.max(1);
			let additional_len = desired_len - self.stack.len();
			self.stack.reserve_exact(additional_len);
		}
		debug_assert!(self.stack.len() < self.limit);
		debug_assert!(self.stack.len() < self.stack.capacity());
		let len = self.stack.len();
		unsafe {
			ptr::write(self.stack.get_unchecked_mut(len), value);
			self.stack.set_len(len + 1);
		}
		Ok(())
	}

	pub fn pop(&mut self) -> Option<T> {
		self.stack.pop()
	}

	/// Remove and Return top element. Does not check for emptyness.
	/// If this is called on a zero length stack, bad things will happen.
	/// Do not call this method unless you can prove the stack has length.
	#[inline]
	pub unsafe fn pop_unchecked(&mut self) -> T {
		debug_assert!(self.stack.len() > 0);
		let len = self.stack.len();
		self.stack.set_len(len - 1);
		ptr::read(self.stack.get_unchecked(self.stack.len()))
	}

	/// Return optional reference to item `depth` distance away from top
	///
	/// `bstack.nth_from_top(0)` gets the top of the stack
	///
	/// `bstack.nth_from_top(1)` gets the item just below the stack
	///
	/// ```
	/// # extern crate wasmi;
	/// # use wasmi::{StackWithLimit, StackSize};
	/// let mut bstack = StackWithLimit::<i32>::with_size(StackSize::from_element_count(2));
	/// bstack.push(4);
	/// assert_eq!(bstack.nth_from_top(0), Some(&4));
	/// assert_eq!(bstack.nth_from_top(1), None);
	/// ```
	pub fn nth_from_top(&self, depth: usize) -> Option<&T> {
		// Be cognizant of integer underflow and overflow here. Both are possible in this situation.
		// len() is unsigned, so if len() == 0, subtraction is a problem
		// depth can legally be 2^32. On a 32 bit system, adding may overflow
		// overflow isn't an issue unless T is zero size
		// In debug builds, underflow panics, but in release mode, underflow is not checked.

		let index = self.stack.len().checked_sub(1)?.checked_sub(depth)?;
		debug_assert!(self.stack.len() > index, "guaranteed by previous line");
		Some(&self.stack[index])
	}

	/// Return mutable reference to item `depth` distance away from top
	///
	/// Does not check whether depth is in range.
	pub fn nth_from_top_mut_unchecked(&mut self, depth: usize) -> &mut T {
		let offset = self.stack.len() - 1 - depth;
		&mut self.stack[offset]
	}

	/// Swaps two elements in the stack.
	///
	/// # Arguments
	///
	/// * a - The index of the first element
	/// * b - The index of the second element
	///
	/// # Panics
	///
	/// Panics if `a` or `b` are out of bound.
	#[inline]
	pub fn swap(&mut self, a: usize, b: usize) {
		self.stack.swap(a, b)
	}

	/// Removes an element from the vector and returns it.
	///
	/// The removed element is replaced by the last element of the vector.
	///
	/// This does not preserve ordering, but is O(1).
	///
	/// # Panics
	///
	/// Panics if `index` is out of bounds.
	pub fn swap_remove(&mut self, index: usize) -> T {
		self.stack.swap_remove(index)
	}

	pub fn top(&self) -> Option<&T> {
		self.stack.last()
	}

	pub fn top_mut(&mut self) -> Option<&mut T> {
		self.stack.last_mut()
	}

	// Same as Vec::[truncate](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.truncate)
	pub fn truncate(&mut self, new_size: usize) {
		self.stack.truncate(new_size)
	}

	#[inline]
	pub fn len(&self) -> usize {
		self.stack.len()
	}

	pub fn is_empty(&self) -> bool {
		self.stack.is_empty()
	}
}

// Why introduce the extra complexity of StackSizeLimit, StackSizeInitial, and StackSize?
// We want to make the user to do the correct thing using the type checker.
// Check out the excellent Rust API Guidelines (linked below) for suggestions
// about type safety in rust.
//
// By introducing the new typed arguments, we turn:
//
// ```
// pub fn new(initial_size: usize, limit: usize) -> StackWithLimit<T> { ... }
// ```
//
// into:
//
// ```
// pub fn new(initial_size: StackSizeInitial<T>, limit: StackSizeLimit<T>) -> StackWithLimit<T> { ... }
// ```
//
// https://rust-lang-nursery.github.io/api-guidelines/type-safety.html#c-custom-type

/// Type for communicating the size of some contigous container.
/// Used for constructing both [`StackSizeLimit`] and [`StackSizeInitial`].
#[derive(Eq, PartialEq, Hash, Debug)]
pub struct StackSize<T> {
	num_elements: usize,
	// PhantomData is a zero-sized type for keeping track of T without rustc complaining.
	phantom: PhantomData<*const T>,
}

impl<T> Clone for StackSize<T> {
	fn clone(&self) -> Self {
		StackSize {
			num_elements: self.num_elements,
			phantom: PhantomData,
		}
	}
}

impl<T> Copy for StackSize<T> {}

impl<T> StackSize<T> {
	/// Create StackSize based on number of elements.
	///
	/// ```
	/// # extern crate wasmi;
	/// # use wasmi::StackSize;
	/// let ss = StackSize::<(u8, u8)>::from_element_count(10);
	/// assert_eq!(ss.element_count(), 10);
	/// ```
	pub fn from_element_count(num_elements: usize) -> StackSize<T> {
		StackSize {
			num_elements,
			phantom: PhantomData,
		}
	}

	/// Compute StackSize based on allowable memory.
	///
	/// ```
	/// # extern crate wasmi;
	/// # use wasmi::StackSize;
	/// let ss = StackSize::<(u8, u8)>::from_byte_count(10);
	/// assert_eq!(ss.element_count(), 10 / 2);
	/// ```
	///
	/// # Errors
	///
	/// In debug mode, panics if size of `T` is 0.
	pub fn from_byte_count(num_bytes: usize) -> StackSize<T> {
		// This debug_assert should catch logical errors.
		debug_assert!(::core::mem::size_of::<T>() != 0, "That doesn't make sense.");

		// In case a zero sized T still makes it into prod. We assume unlimited stack
		// size instead of panicking.
		let element_count = if ::core::mem::size_of::<T>() != 0 {
			num_bytes / ::core::mem::size_of::<T>()
		} else {
			usize::MAX // Semi-relevant fun fact: Vec::<()>::new().capacity() == usize::MAX
		};

		StackSize::from_element_count(element_count)
	}

	/// Return number the of elements this StackSize indicates.
	///
	/// ```
	/// # use wasmi::StackSize;
	/// let ss = StackSize::<(u8, u8)>::from_element_count(10);
	/// assert_eq!(ss.element_count(), 10);
	/// ```
	///
	pub fn element_count(&self) -> usize {
		self.num_elements
	}

	/// Create StackSizeLimit out of self
	///
	/// ```
	/// # use wasmi::{StackSize, StackSizeLimit, RuntimeValue};
	/// let values_limit: StackSizeLimit<RuntimeValue> = StackSize::from_element_count(1024).into_limit();
	/// ```
	pub fn into_limit(self) -> StackSizeLimit<T> {
		StackSizeLimit(self)
	}

	/// Create StackSizeLimit out of self
	pub fn into_initial(self) -> StackSizeInitial<T> {
		StackSizeInitial(self)
	}

	/// Create StackSizeLimit with no upper bound.
	pub fn unlimited() -> StackSizeLimit<T> {
		StackSize::from_element_count(usize::MAX).into_limit()
	}
}

/// Max size a stack may become.
///
/// Constructed by [`StackSize::into_limit`](into_limit) or [`StackSize::unlimited`](unlimited)
///
/// [into_limit]: type.StackSize.into_limit.html
/// [unlimited]: type.StackSize.unlimited.html
pub struct StackSizeLimit<T>(StackSize<T>);

/// Number of pre-allocated elements.
///
/// Constructed by [`StackSize::into_initial`]
///
/// [into_initial]: type.StackSize.into_initial.html
pub struct StackSizeInitial<T>(StackSize<T>);

#[cfg(test)]
mod test {
	use super::{StackSize, StackSizeInitial, StackSizeLimit, StackWithLimit};
	use core::usize;

	#[test]
	fn nth_from_top() {
		let mut bstack = StackWithLimit::<i32>::with_size(StackSize::from_element_count(2));
		assert_eq!(bstack.nth_from_top(0), None);
		bstack.push(1).unwrap();
		bstack.push(2).unwrap();
		bstack.push(3).unwrap_err();
		assert_eq!(bstack.nth_from_top(0), Some(&2));
		assert_eq!(bstack.nth_from_top(1), Some(&1));
		assert_eq!(bstack.nth_from_top(2), None);
		assert_eq!(bstack.nth_from_top(3), None);
	}

	fn exersize(mut bstack: StackWithLimit<i32>) {
		assert!(bstack.is_empty());
		assert_eq!(bstack.len(), 0);
		assert_eq!(bstack.top(), None);
		assert_eq!(bstack.top_mut(), None);
		assert_eq!(bstack.pop(), None);
		bstack.push(0).unwrap();
		assert!(!bstack.is_empty());
		assert_eq!(bstack.len(), 1);
		assert_eq!(bstack.top(), Some(&0));
		assert_eq!(bstack.top_mut(), Some(&mut 0));
		assert_eq!(bstack.pop(), Some(0));

		bstack.push(0).unwrap();
		bstack.push(0).unwrap();
		bstack.push(0).unwrap();
		bstack.push(0).unwrap();
		assert_eq!(bstack.len(), 4);
		bstack.truncate(8);
		assert_eq!(bstack.len(), 4);
		bstack.truncate(4);
		assert_eq!(bstack.len(), 4);
		bstack.truncate(2);
		assert_eq!(bstack.len(), 2);
		bstack.truncate(0);
		assert_eq!(bstack.len(), 0);
	}

	#[test]
	fn stack_with_limit() {
		let bstack = StackWithLimit::<i32>::with_size(StackSize::from_element_count(20));
		exersize(bstack);
	}

	// Check for integer overflow bugs
	#[test]
	fn practically_unlimited_stack() {
		let bstack = StackWithLimit::<i32>::new(
			StackSizeInitial(StackSize::from_element_count(0)),
			StackSizeLimit(StackSize::from_element_count(usize::MAX)),
		);

		exersize(bstack);
	}

	// Make sure the stack resizes properly.
	#[test]
	fn must_resize() {
		let mut bstack = StackWithLimit::<i32>::new(
			StackSizeInitial(StackSize::from_element_count(2)),
			StackSizeLimit(StackSize::from_element_count(4)),
		);
		bstack.push(2).unwrap();
		bstack.push(4).unwrap();
		bstack.push(8).unwrap();
		bstack.push(16).unwrap();
		bstack.push(16).unwrap_err();
		assert_eq!(bstack.pop(), Some(16));
		assert_eq!(bstack.pop(), Some(8));
		assert_eq!(bstack.pop(), Some(4));
		assert_eq!(bstack.pop(), Some(2));
	}

	#[test]
	fn pop_unchecked() {
		let mut bstack = StackWithLimit::<i32>::with_size(StackSize::from_element_count(20));
		bstack.push(8).unwrap();
		bstack.push(0).unwrap();
		assert_eq!(unsafe { bstack.pop_unchecked() }, 0);
		assert_eq!(unsafe { bstack.pop_unchecked() }, 8);
	}
}
