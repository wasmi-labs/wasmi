//! Definitions for visualization of generic utility components.

use core::{fmt, fmt::Display};

/// The enclosure style for a [`DisplaySlice`] and [`DisplaySequence`].
#[derive(Debug, Copy, Clone)]
pub enum EnclosureStyle {
    /// `(` and `)`
    Paren,
    /// `[` and `]`
    Bracket,
    /// `{` and `}`
    #[allow(dead_code)] // TODO: unsilence warning
    Brace,
}

impl EnclosureStyle {
    /// Returns the opening enclosure.
    pub fn open(self) -> &'static str {
        match self {
            Self::Paren => "(",
            Self::Bracket => "[",
            Self::Brace => "{",
        }
    }

    /// Returns the closing enclosure.
    pub fn close(self) -> &'static str {
        match self {
            Self::Paren => ")",
            Self::Bracket => "]",
            Self::Brace => "}",
        }
    }
}

/// The enclosure strategy for a [`DisplaySlice`] and [`DisplaySequence`].
#[derive(Debug, Copy, Clone)]
pub enum Enclosure {
    Always(EnclosureStyle),
    NoSingle(EnclosureStyle),
    #[allow(dead_code)] // TODO: unsilence warning
    Never,
}

impl Enclosure {
    /// Create [`Enclosure`] that always displays.
    pub fn always(style: EnclosureStyle) -> Self {
        Self::Always(style)
    }

    /// Create [`Enclosure`] that never displays.
    #[allow(dead_code)] // TODO: unsilence warning
    pub fn never() -> Self {
        Self::Never
    }

    /// Create [`Enclosure`] that does not display for single items.
    pub fn no_single(style: EnclosureStyle) -> Self {
        Self::NoSingle(style)
    }

    /// Returns the [`EnclosureStyle`] of the [`Enclosure`].
    pub fn style(self) -> Option<EnclosureStyle> {
        match self {
            Enclosure::Always(style) => Some(style),
            Enclosure::NoSingle(style) => Some(style),
            Enclosure::Never => None,
        }
    }
}

/// Displays the slice in a human readable form.
///
/// # Note
///
/// Single element slices just displayed their single elemment as usual.
/// Empty slices are written as `[]`.
/// Normal slices print as `Debug` but with their elements as `Display`.
pub struct DisplaySlice<'a, T> {
    /// The enclosure style.
    enclosure: Enclosure,
    /// The displayed slice.
    slice: &'a [T],
}

impl<'a, T> DisplaySlice<'a, T> {
    /// Creates a new [`DisplaySlice`] for the given `slice` with `enclosure` style.
    pub fn new(enclosure: Enclosure, slice: &'a [T]) -> Self {
        Self { enclosure, slice }
    }
}

impl<T> Display for DisplaySlice<'_, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let style = match self.enclosure {
            Enclosure::Always(style) => Some(style),
            Enclosure::NoSingle(style) => Some(style).filter(|_| self.slice.len() != 1),
            Enclosure::Never => None,
        };
        let open = style.map(EnclosureStyle::open).unwrap_or("");
        let close = style.map(EnclosureStyle::close).unwrap_or("");
        write!(f, "{open}")?;
        if let Some((first, rest)) = self.slice.split_first() {
            write!(f, "{}", first)?;
            for elem in rest {
                write!(f, ", {}", elem)?;
            }
        }
        write!(f, "{close}")?;
        Ok(())
    }
}

/// Displays the iterator in a human readable form.
///
/// # Note
///
/// Read [`DisplaySlice`] documentation to see how iterators are visualized.
pub struct DisplaySequence<T> {
    /// The enclosure style.
    enclosure: Enclosure,
    /// The displayed items.
    items: T,
}

impl<T> DisplaySequence<T> {
    /// Creates a new [`DisplaySlice`] for the given `slice` with `enclosure` style.
    pub fn new(enclosure: Enclosure, items: T) -> Self {
        Self { enclosure, items }
    }
}

impl<T, V> Display for DisplaySequence<T>
where
    T: Iterator<Item = V> + Clone,
    V: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut iter = self.items.clone();
        let style = self.enclosure.style();
        let open = style.map(EnclosureStyle::open).unwrap_or("");
        let close = style.map(EnclosureStyle::close).unwrap_or("");
        match (iter.next(), iter.next()) {
            (None, _) => {
                write!(f, "{open}{close}")
            }
            (Some(single), None) => {
                if matches!(self.enclosure, Enclosure::Always(_)) {
                    write!(f, "{open}{single}{close}")
                } else {
                    write!(f, "{single}")
                }
            }
            (Some(fst), Some(snd)) => {
                write!(f, "{open}{fst}, {snd}")?;
                for next in iter {
                    write!(f, ", {next}")?;
                }
                write!(f, "{close}")
            }
        }
    }
}
