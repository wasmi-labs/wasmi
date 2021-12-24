use crate::{func::FuncRef, module::check_limits, Error};
use alloc::{rc::Rc, vec::Vec};
use core::{cell::RefCell, fmt, u32};
use parity_wasm::elements::ResizableLimits;

/// Reference to a table (See [`TableInstance`] for details).
///
/// This reference has a reference-counting semantics.
///
/// [`TableInstance`]: struct.TableInstance.html
///
#[derive(Clone, Debug)]
pub struct TableRef(Rc<TableInstance>);

impl ::core::ops::Deref for TableRef {
    type Target = TableInstance;
    fn deref(&self) -> &TableInstance {
        &self.0
    }
}

/// Runtime representation of a table.
///
/// A table is a array of untyped functions. It allows wasm code to call functions
/// indirectly through a dynamic index into a table. For example, this allows emulating function
/// pointers by way of table indices.
///
/// Table is created with an initial size but can be grown dynamically via [`grow`] method.
/// Growth can be limited by an optional maximum size.
///
/// In future, a table might be extended to be able to hold not only functions but different types.
///
/// [`grow`]: #method.grow
///
pub struct TableInstance {
    /// Table limits.
    limits: ResizableLimits,
    /// Table memory buffer.
    buffer: RefCell<Vec<Option<FuncRef>>>,
}

impl fmt::Debug for TableInstance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("TableInstance")
            .field("limits", &self.limits)
            .field("buffer.len", &self.buffer.borrow().len())
            .finish()
    }
}

impl TableInstance {
    /// Allocate a table instance.
    ///
    /// The table allocated with initial size, specified by `initial_size`.
    /// Maximum size can be specified by `maximum_size`.
    ///
    /// All table elements are allocated uninitialized.
    ///
    /// # Errors
    ///
    /// Returns `Err` if `initial_size` is greater than `maximum_size`.
    pub fn alloc(initial_size: u32, maximum_size: Option<u32>) -> Result<TableRef, Error> {
        let table = TableInstance::new(ResizableLimits::new(initial_size, maximum_size))?;
        Ok(TableRef(Rc::new(table)))
    }

    fn new(limits: ResizableLimits) -> Result<TableInstance, Error> {
        check_limits(&limits)?;
        Ok(TableInstance {
            buffer: RefCell::new(vec![None; limits.initial() as usize]),
            limits,
        })
    }

    /// Return table limits.
    pub(crate) fn limits(&self) -> &ResizableLimits {
        &self.limits
    }

    /// Returns size this table was created with.
    pub fn initial_size(&self) -> u32 {
        self.limits.initial()
    }

    /// Returns maximum size `TableInstance` can grow to.
    pub fn maximum_size(&self) -> Option<u32> {
        self.limits.maximum()
    }

    /// Returns current size of the table.
    pub fn current_size(&self) -> u32 {
        self.buffer.borrow().len() as u32
    }

    /// Increases the size of the table by given number of elements.
    ///
    /// # Errors
    ///
    /// Returns `Err` if tried to allocate more elements than permited by limit.
    pub fn grow(&self, by: u32) -> Result<(), Error> {
        let mut buffer = self.buffer.borrow_mut();
        let maximum_size = self.maximum_size().unwrap_or(u32::MAX);
        let new_size = self
            .current_size()
            .checked_add(by)
            .and_then(|new_size| {
                if maximum_size < new_size {
                    None
                } else {
                    Some(new_size)
                }
            })
            .ok_or_else(|| {
                Error::Table(format!(
                    "Trying to grow table by {} items when there are already {} items",
                    by,
                    self.current_size(),
                ))
            })?;
        buffer.resize(new_size as usize, None);
        Ok(())
    }

    /// Get the specific value in the table
    pub fn get(&self, offset: u32) -> Result<Option<FuncRef>, Error> {
        let buffer = self.buffer.borrow();
        let buffer_len = buffer.len();
        let table_elem = buffer.get(offset as usize).cloned().ok_or_else(|| {
            Error::Table(format!(
                "trying to read table item with index {} when there are only {} items",
                offset, buffer_len
            ))
        })?;
        Ok(table_elem)
    }

    /// Set the table element to the specified function.
    pub fn set(&self, offset: u32, value: Option<FuncRef>) -> Result<(), Error> {
        let mut buffer = self.buffer.borrow_mut();
        let buffer_len = buffer.len();
        let table_elem = buffer.get_mut(offset as usize).ok_or_else(|| {
            Error::Table(format!(
                "trying to update table item with index {} when there are only {} items",
                offset, buffer_len
            ))
        })?;
        *table_elem = value;
        Ok(())
    }
}
