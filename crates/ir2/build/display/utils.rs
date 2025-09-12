use core::fmt::{self, Display};

#[derive(Copy, Clone, Default)]
pub struct Indent(usize);

impl Indent {
    pub fn inc(self) -> Self {
        Self(self.0 + 1)
    }

    pub fn inc_by(self, delta: usize) -> Self {
        Self(self.0 + delta)
    }
}

impl Display for Indent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for _ in 0..self.0 {
            f.write_str("    ")?;
        }
        Ok(())
    }
}

pub struct DisplayConcat<T>(pub T);

impl<T0, T1> Display for DisplayConcat<(T0, T1)>
where
    T0: Display,
    T1: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (t0, t1) = &self.0;
        write!(f, "{t0}{t1}")
    }
}

impl<T0, T1, T2> Display for DisplayConcat<(T0, T1, T2)>
where
    T0: Display,
    T1: Display,
    T2: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (t0, t1, t2) = &self.0;
        write!(f, "{t0}{t1}{t2}")
    }
}

pub struct DisplaySequence<I, S> {
    iter: I,
    sep: S,
}

impl<I, S> DisplaySequence<I, S> {
    pub fn new(sep: S, iter: I) -> Self {
        Self { sep, iter }
    }
}

impl<I, S> Display for DisplaySequence<I, S>
where
    I: IntoIterator<Item: Display> + Clone,
    S: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut iter = self.iter.clone().into_iter();
        let Some(first) = iter.next() else {
            return Ok(());
        };
        first.fmt(f)?;
        let sep = &self.sep;
        for item in iter {
            write!(f, "{sep}{item}")?;
        }
        Ok(())
    }
}

pub enum DisplayMaybe<T> {
    Some(T),
    None,
}

impl<T> Display for DisplayMaybe<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let DisplayMaybe::Some(field) = self {
            field.fmt(f)?;
        }
        Ok(())
    }
}

pub trait IntoDisplayMaybe<T> {
    fn display_maybe(self) -> DisplayMaybe<T>;
}
impl<T> IntoDisplayMaybe<T> for Option<T> {
    fn display_maybe(self) -> DisplayMaybe<T> {
        DisplayMaybe::from(self)
    }
}

impl<T> From<T> for DisplayMaybe<T> {
    fn from(value: T) -> Self {
        DisplayMaybe::Some(value)
    }
}

impl<T> From<Option<T>> for DisplayMaybe<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => Self::Some(value),
            None => Self::None,
        }
    }
}
