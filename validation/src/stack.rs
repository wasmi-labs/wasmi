use alloc::{string::String, vec::Vec};

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
            limit,
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
