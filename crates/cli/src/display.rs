use std::fmt::{self, Display};
use wasmi::{FuncType, Value};

/// Wrapper type that implements `Display` for [`Value`].
pub struct DisplayValue<'a>(&'a Value);

impl<'a> From<&'a Value> for DisplayValue<'a> {
    fn from(value: &'a Value) -> Self {
        Self(value)
    }
}

impl<'a> fmt::Display for DisplayValue<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Value::I32(value) => write!(f, "{value}"),
            Value::I64(value) => write!(f, "{value}"),
            Value::F32(value) => write!(f, "{value}"),
            Value::F64(value) => write!(f, "{value}"),
            Value::FuncRef(value) => panic!("cannot display funcref values but found {value:?}"),
            Value::ExternRef(value) => {
                panic!("cannot display externref values but found {value:?}")
            }
        }
    }
}

/// Wrapper type around [`FuncType`] that implements `Display` for it.
pub struct DisplayFuncType<'a>(&'a FuncType);

impl<'a> From<&'a FuncType> for DisplayFuncType<'a> {
    fn from(func_type: &'a FuncType) -> Self {
        Self(func_type)
    }
}

impl fmt::Display for DisplayFuncType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fn(")?;
        let params = self.0.params();
        let results = self.0.results();
        write!(f, "{}", DisplaySequence::new(", ", params))?;
        write!(f, ")")?;
        if let Some((first, rest)) = results.split_first() {
            write!(f, " -> ")?;
            if results.len() == 1 {
                write!(f, "{}", &results[0])?;
            } else {
                write!(f, "({})", DisplaySequence::new(", ", results))?;
            }
        }
        Ok(())
    }
}

/// Display-wrapper for generic sequences with separators.
#[derive(Debug)]
pub struct DisplaySequence<'a, T> {
    /// The sequence to display.
    sequence: Box<[T]>,
    /// The separator between the displayed sequence items.
    separator: &'a str,
}

impl<'a, T> DisplaySequence<'a, T> {
    /// Creates a new [`DisplaySequence`] for the given `separator` and `sequence` iterator.
    pub fn new<I>(separator: &'a str, sequence: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let sequence = sequence.into_iter().collect();
        Self {
            sequence,
            separator,
        }
    }
}

impl<T> Display for DisplaySequence<'_, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let separator = self.separator;
        if let Some((first, rest)) = self.sequence.split_first() {
            write!(f, "{first}")?;
            for param in rest {
                write!(f, "{separator}{param}")?;
            }
        }
        Ok(())
    }
}
