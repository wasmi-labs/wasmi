use core::fmt::{self, Display};

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

pub struct DisplaySequence<T>(pub T);

impl<T> Display for DisplaySequence<T>
where
    T: IntoIterator<Item: Display> + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in self.0.clone() {
            write!(f, "{item}")?;
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
