use std::u32;
use std::ops::Range;
use std::cmp;
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use parity_wasm::elements::ResizableLimits;
use Error;
use memory_units::{RoundUpTo, Pages, Bytes};

/// Size of a page of [linear memory][`MemoryInstance`] - 64KiB.
///
/// The size of a memory is always a integer multiple of a page size.
///
/// [`MemoryInstance`]: struct.MemoryInstance.html
pub const LINEAR_MEMORY_PAGE_SIZE: Bytes = Bytes(65536);

/// Maximal number of pages.
const LINEAR_MEMORY_MAX_PAGES: Pages = Pages(65536);

/// Reference to a memory (See [`MemoryInstance`] for details).
///
/// This reference has a reference-counting semantics.
///
/// [`MemoryInstance`]: struct.MemoryInstance.html
///
#[derive(Clone, Debug)]
pub struct MemoryRef(Rc<MemoryInstance>);

impl ::std::ops::Deref for MemoryRef {
	type Target = MemoryInstance;
	fn deref(&self) -> &MemoryInstance {
		&self.0
	}
}

/// Runtime representation of a linear memory (or `memory` for short).
///
/// A memory is a contiguous, mutable array of raw bytes. Wasm code can load and store values
/// from/to a linear memory at any byte address.
/// A trap occurs if an access is not within the bounds of the current memory size.
///
/// A memory is created with an initial size but can be grown dynamically.
/// The growth can be limited by specifying maximum size.
/// The size of a memory is always a integer multiple of a [page size][`LINEAR_MEMORY_PAGE_SIZE`] - 64KiB.
///
/// At the moment, wasm doesn't provide any way to shrink the memory.
///
/// [`LINEAR_MEMORY_PAGE_SIZE`]: constant.LINEAR_MEMORY_PAGE_SIZE.html
pub struct MemoryInstance {
	/// Memofy limits.
	limits: ResizableLimits,
	/// Linear memory buffer.
	buffer: RefCell<Vec<u8>>,
	initial: Pages,
	maximum: Option<Pages>,
}

impl fmt::Debug for MemoryInstance {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("MemoryInstance")
			.field("limits", &self.limits)
			.field("buffer.len", &self.buffer.borrow().len())
			.field("maximum", &self.maximum)
			.field("initial", &self.initial)
			.finish()
	}
}

struct CheckedRegion<'a, B: 'a> where B: ::std::ops::Deref<Target=Vec<u8>> {
	buffer: &'a B,
	offset: usize,
	size: usize,
}

impl<'a, B: 'a> CheckedRegion<'a, B> where B: ::std::ops::Deref<Target=Vec<u8>> {
	fn range(&self) -> Range<usize> {
		self.offset..self.offset+self.size
	}

	fn slice(&self) -> &[u8] {
		&self.buffer[self.range()]
	}

	fn intersects(&self, other: &Self) -> bool {
		let low = cmp::max(self.offset, other.offset);
		let high = cmp::min(self.offset + self.size, other.offset + other.size);

		low < high
	}
}

impl MemoryInstance {
	/// Allocate a memory instance.
	///
	/// The memory allocated with initial number of pages specified by `initial`.
	/// Minimal possible value for `initial` is 0 and maximum possible is `65536`.
	/// (Since maximum addressible memory is 2<sup>32</sup> = 4GiB = 65536 * [64KiB][`LINEAR_MEMORY_PAGE_SIZE`]).
	///
	/// It is possible to limit maximum number of pages this memory instance can have by specifying
	/// `maximum`. If not specified, this memory instance would be able to allocate up to 4GiB.
	///
	/// Allocated memory is always zeroed.
	///
	/// # Errors
	///
	/// Returns `Err` if:
	///
	/// - `initial` is greater than `maximum`
	/// - either `initial` or `maximum` is greater than `65536`.
	///
	/// [`LINEAR_MEMORY_PAGE_SIZE`]: constant.LINEAR_MEMORY_PAGE_SIZE.html
	pub fn alloc(initial: Pages, maximum: Option<Pages>) -> Result<MemoryRef, Error> {
		validate_memory(initial, maximum).map_err(Error::Memory)?;

		let memory = MemoryInstance::new(initial, maximum);
		Ok(MemoryRef(Rc::new(memory)))
	}

	/// Create new linear memory instance.
	fn new(initial: Pages, maximum: Option<Pages>) -> Self {
		let limits = ResizableLimits::new(initial.0 as u32, maximum.map(|p| p.0 as u32));

		let initial_size: Bytes = initial.into();
		MemoryInstance {
			limits: limits,
			buffer: RefCell::new(vec![0; initial_size.0]),
			initial: initial,
			maximum: maximum,
		}
	}

	/// Return linear memory limits.
	pub(crate) fn limits(&self) -> &ResizableLimits {
		&self.limits
	}

	/// Returns number of pages this `MemoryInstance` was created with.
	pub fn initial(&self) -> Pages {
		self.initial
	}

	/// Returns maximum amount of pages this `MemoryInstance` can grow to.
	///
	/// Returns `None` if there is no limit set.
	/// Maximum memory size cannot exceed `65536` pages or 4GiB.
	pub fn maximum(&self) -> Option<Pages> {
		self.maximum
	}

	/// Returns current linear memory size.
	///
	/// Maximum memory size cannot exceed `65536` pages or 4GiB.
	///
	/// # Example
	///
	/// To convert number of pages to number of bytes you can use the following code:
	///
	/// ```rust
	/// use wasmi::MemoryInstance;
	/// use wasmi::memory_units::*;
	///
	/// let memory = MemoryInstance::alloc(Pages(1), None).unwrap();
	/// let byte_size: Bytes = memory.current_size().into();
	/// assert_eq!(
	///     byte_size,
	///     Bytes(65536),
	/// );
	/// ```
	pub fn current_size(&self) -> Pages {
		Bytes(self.buffer.borrow().len()).round_up_to()
	}

	/// Copy data from memory at given offset.
	///
	/// This will allocate vector for you.
	/// If you can provide a mutable slice you can use [`get_into`].
	///
	/// [`get_into`]: #method.get_into
	pub fn get(&self, offset: u32, size: usize) -> Result<Vec<u8>, Error> {
		let buffer = self.buffer.borrow();
		let region = self.checked_region(&buffer, offset as usize, size)?;

		Ok(region.slice().to_vec())
	}

	/// Copy data from given offset in the memory into `target` slice.
	///
	/// # Errors
	///
	/// Returns `Err` if the specified region is out of bounds.
	pub fn get_into(&self, offset: u32, target: &mut [u8]) -> Result<(), Error> {
		let buffer = self.buffer.borrow();
		let region = self.checked_region(&buffer, offset as usize, target.len())?;

		target.copy_from_slice(region.slice());

		Ok(())
	}

	/// Copy data in the memory at given offset.
	pub fn set(&self, offset: u32, value: &[u8]) -> Result<(), Error> {
		let mut buffer = self.buffer.borrow_mut();
		let range = self.checked_region(&buffer, offset as usize, value.len())?.range();

		buffer[range].copy_from_slice(value);

		Ok(())
	}

	/// Increases the size of the linear memory by given number of pages.
	/// Returns previous memory size if succeeds.
	///
	/// # Errors
	///
	/// Returns `Err` if attempted to allocate more memory than permited by the limit.
	pub fn grow(&self, additional: Pages) -> Result<Pages, Error> {
		let size_before_grow: Pages = self.current_size();

		if additional == Pages(0) {
			return Ok(size_before_grow);
		}
		if additional > Pages(65536) {
			return Err(Error::Memory(format!(
				"Trying to grow memory by more than 65536 pages"
			)));
		}

		let new_size: Pages = size_before_grow + additional;
		let maximum = self.maximum.unwrap_or(LINEAR_MEMORY_MAX_PAGES);
		if new_size > maximum {
			return Err(Error::Memory(format!(
				"Trying to grow memory by {} pages when already have {}",
				additional.0, size_before_grow.0,
			)));
		}

		// Resize underlying buffer up to a new size filling newly allocated space with zeroes.
		// This size is guaranteed to be larger than current size.
		let new_buffer_length: Bytes = new_size.into();
		{
			let mut buffer = self.buffer.borrow_mut();
			debug_assert!(new_buffer_length.0 > buffer.len());
			buffer.resize(new_buffer_length.0, 0);
		}
		Ok(size_before_grow)
	}

	fn checked_region<'a, B>(&self, buffer: &'a B, offset: usize, size: usize) -> Result<CheckedRegion<'a, B>, Error>
		where B: ::std::ops::Deref<Target=Vec<u8>>
	{
		let end = offset.checked_add(size)
			.ok_or_else(|| Error::Memory(format!("trying to access memory block of size {} from offset {}", size, offset)))?;

		if end > buffer.len() {
			return Err(Error::Memory(format!("trying to access region [{}..{}] in memory [0..{}]", offset, end, buffer.len())));
		}

		Ok(CheckedRegion {
			buffer: buffer,
			offset: offset,
			size: size,
		})
	}

	/// Copy contents of one memory region to another.
	///
	/// Semantically equivalent to `memmove`.
	///
	/// # Errors
	///
	/// Returns `Err` if either of specified regions is out of bounds.
	pub fn copy(&self, src_offset: usize, dst_offset: usize, len: usize) -> Result<(), Error> {
		let buffer = self.buffer.borrow_mut();

		let read_region = self.checked_region(&buffer, src_offset, len)?;
		let write_region = self.checked_region(&buffer, dst_offset, len)?;

		unsafe { ::std::ptr::copy(
			buffer[read_region.range()].as_ptr(),
			buffer[write_region.range()].as_ptr() as *mut _,
			len,
		)}

		Ok(())
	}

	/// Copy contents of one memory region to another (non-overlapping version).
	///
	/// Semantically equivalent to `memcpy`.
	/// but returns Error if source overlaping with destination.
	///
	/// # Errors
	///
	/// Returns `Err` if:
	///
	/// - either of specified regions is out of bounds,
	/// - these regions overlaps.
	pub fn copy_nonoverlapping(&self, src_offset: usize, dst_offset: usize, len: usize) -> Result<(), Error> {
		let buffer = self.buffer.borrow_mut();

		let read_region = self.checked_region(&buffer, src_offset, len)?;
		let write_region = self.checked_region(&buffer, dst_offset, len)?;

		if read_region.intersects(&write_region) {
			return Err(Error::Memory(format!("non-overlapping copy is used for overlapping regions")))
		}

		unsafe { ::std::ptr::copy_nonoverlapping(
			buffer[read_region.range()].as_ptr(),
			buffer[write_region.range()].as_ptr() as *mut _,
			len,
		)}

		Ok(())
	}

	/// Fill memory region with a specified value.
	///
	/// Semantically equivalent to `memset`.
	///
	/// # Errors
	///
	/// Returns `Err` if the specified region is out of bounds.
	pub fn clear(&self, offset: usize, new_val: u8, len: usize) -> Result<(), Error> {
		let mut buffer = self.buffer.borrow_mut();

		let range = self.checked_region(&buffer, offset, len)?.range();
		for val in &mut buffer[range] { *val = new_val }
		Ok(())
	}

	/// Fill specified memory region with zeroes.
	///
	/// # Errors
	///
	/// Returns `Err` if the specified region is out of bounds.
	pub fn zero(&self, offset: usize, len: usize) -> Result<(), Error> {
		self.clear(offset, 0, len)
	}
}

pub fn validate_memory(initial: Pages, maximum: Option<Pages>) -> Result<(), String> {
	if initial > LINEAR_MEMORY_MAX_PAGES {
		return Err(format!("initial memory size must be at most {} pages", LINEAR_MEMORY_MAX_PAGES.0));
	}
	if let Some(maximum) = maximum {
		if initial > maximum {
			return Err(format!(
				"maximum limit {} is less than minimum {}",
				maximum.0,
				initial.0,
			));
		}

		if maximum > LINEAR_MEMORY_MAX_PAGES {
			return Err(format!("maximum memory size must be at most {} pages", LINEAR_MEMORY_MAX_PAGES.0));
		}
	}
	Ok(())
}

#[cfg(test)]
mod tests {

	use super::{MemoryInstance, LINEAR_MEMORY_PAGE_SIZE};
	use Error;
	use memory_units::Pages;

	#[test]
	fn alloc() {
		let fixtures = &[
			(0, None, true),
			(0, Some(0), true),
			(1, None, true),
			(1, Some(1), true),
			(0, Some(1), true),
			(1, Some(0), false),
			(0, Some(65536), true),
			(65536, Some(65536), true),
			(65536, Some(0), false),
			(65536, None, true),
		];
		for (index, &(initial, maybe_max, expected_ok)) in fixtures.iter().enumerate() {
			let initial: Pages = Pages(initial);
			let maximum: Option<Pages> = maybe_max.map(|m| Pages(m));
			let result = MemoryInstance::alloc(initial, maximum);
			if result.is_ok() != expected_ok {
				panic!(
					"unexpected error at {}, initial={:?}, max={:?}, expected={}, result={:?}",
					index,
					initial,
					maybe_max,
					expected_ok,
					result,
				);
			}
		}
	}

	#[test]
	fn ensure_page_size() {
		use memory_units::ByteSize;
		assert_eq!(LINEAR_MEMORY_PAGE_SIZE, Pages::byte_size());
	}

	fn create_memory(initial_content: &[u8]) -> MemoryInstance {
		let mem = MemoryInstance::new(Pages(1), Some(Pages(1)));
		mem.set(0, initial_content).expect("Successful initialize the memory");
		mem
	}

	#[test]
	fn copy_overlaps_1() {
		let mem = create_memory(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
		mem.copy(0, 4, 6).expect("Successfully copy the elements");
		let result = mem.get(0, 10).expect("Successfully retrieve the result");
		assert_eq!(result, &[0, 1, 2, 3, 0, 1, 2, 3, 4, 5]);
	}

	#[test]
	fn copy_overlaps_2() {
		let mem = create_memory(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
		mem.copy(4, 0, 6).expect("Successfully copy the elements");
		let result = mem.get(0, 10).expect("Successfully retrieve the result");
		assert_eq!(result, &[4, 5, 6, 7, 8, 9, 6, 7, 8, 9]);
	}

	#[test]
	fn copy_nonoverlapping() {
		let mem = create_memory(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
		mem.copy_nonoverlapping(0, 10, 10).expect("Successfully copy the elements");
		let result = mem.get(10, 10).expect("Successfully retrieve the result");
		assert_eq!(result, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
	}

	#[test]
	fn copy_nonoverlapping_overlaps_1() {
		let mem = create_memory(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
		let result = mem.copy_nonoverlapping(0, 4, 6);
		match result {
			Err(Error::Memory(_)) => {},
			_ => panic!("Expected Error::Memory(_) result, but got {:?}", result),
		}
	}

	#[test]
	fn copy_nonoverlapping_overlaps_2() {
		let mem = create_memory(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
		let result = mem.copy_nonoverlapping(4, 0, 6);
		match result {
			Err(Error::Memory(_)) => {},
			_ => panic!("Expected Error::Memory(_), but got {:?}", result),
		}
	}

	#[test]
	fn clear() {
		let mem = create_memory(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
		mem.clear(0, 0x4A, 10).expect("To successfully clear the memory");
		let result = mem.get(0, 10).expect("To successfully retrieve the result");
		assert_eq!(result, &[0x4A; 10]);
	}

	#[test]
	fn get_into() {
		let mem = MemoryInstance::new(Pages(1), None);
		mem.set(6, &[13, 17, 129]).expect("memory set should not fail");

		let mut data = [0u8; 2];
		mem.get_into(7, &mut data[..]).expect("get_into should not fail");

		assert_eq!(data, [17, 129]);
	}
}
