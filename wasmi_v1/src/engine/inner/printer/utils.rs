//! Definitions for visualization of generic utility components.

use core::{fmt, fmt::Display};

/// Displays the slice in a human readable form.
///
/// # Note
///
/// Single element slices just displayed their single elemment as usual.
/// Empty slices are written as `[]`.
/// Normal slices print as `Debug` but with their elements as `Display`.
pub struct DisplaySlice<'a, T>(&'a [T]);

impl<'a, T> From<&'a [T]> for DisplaySlice<'a, T> {
    fn from(slice: &'a [T]) -> Self {
        Self(slice)
    }
}

impl<T> Display for DisplaySlice<'_, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let is_list = self.0.len() != 1;
        if is_list {
            write!(f, "[")?;
        }
        if let Some((first, rest)) = self.0.split_first() {
            write!(f, "{}", first)?;
            for elem in rest {
                write!(f, ", {}", elem)?;
            }
        }
        if is_list {
            write!(f, "]")?;
        }
        Ok(())
    }
}

/// Displays the iterator in a human readable form.
///
/// # Note
///
/// Read [`DisplaySlice`] documentation to see how iterators are visualized.
pub struct DisplaySequence<T> {
    items: T,
}

impl<T> From<T> for DisplaySequence<T> {
    fn from(items: T) -> Self {
        Self { items }
    }
}

impl<T, V> Display for DisplaySequence<T>
where
    T: Iterator<Item = V> + Clone,
    V: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut iter = self.items.clone();
        match (iter.next(), iter.next()) {
            (None, _) => write!(f, "[]"),
            (Some(single), None) => write!(f, "{single}"),
            (Some(fst), Some(snd)) => {
                write!(f, "[{fst}, {snd}")?;
                for next in iter {
                    write!(f, ", {next}")?;
                }
                write!(f, "]")?;
                Ok(())
            }
        }
    }
}
