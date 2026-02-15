use crate::context::Context;
use std::fmt::{self, Display};
use wasmi::{FuncType, Val, ValType};

/// [`Display`]-wrapper type for [`ValType`].
pub struct DisplayValueType<'a>(&'a ValType);

impl<'a> From<&'a ValType> for DisplayValueType<'a> {
    fn from(value_type: &'a ValType) -> Self {
        Self(value_type)
    }
}

impl Display for DisplayValueType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self.0 {
            ValType::I32 => "i32",
            ValType::I64 => "i64",
            ValType::F32 => "f32",
            ValType::F64 => "f64",
            ValType::V128 => "v128",
            ValType::FuncRef => "funcref",
            ValType::ExternRef => "externref",
        };
        f.write_str(s)
    }
}

/// [`Display`]-wrapper type for [`Val`].
pub struct DisplayValue<'a>(&'a Val);

impl<'a> From<&'a Val> for DisplayValue<'a> {
    fn from(value: &'a Val) -> Self {
        Self(value)
    }
}

impl fmt::Display for DisplayValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Val::I32(value) => value.fmt(f),
            Val::I64(value) => value.fmt(f),
            Val::F32(value) => value.fmt(f),
            Val::F64(value) => value.fmt(f),
            Val::V128(value) => {
                write!(f, "0x{:032X}", value.as_u128())
            }
            Val::FuncRef(_) | Val::ExternRef(_) => {
                panic!("cannot display reference values but found {:?}", self.0)
            }
        }
    }
}

/// [`Display`]-wrapper type around [`FuncType`].
pub struct DisplayFuncType<'a> {
    name: Option<&'a str>,
    func_type: &'a FuncType,
}

impl<'a> DisplayFuncType<'a> {
    /// Creates a named [`DisplayFuncType`] for the given [`FuncType`].
    pub fn new(name: &'a str, func_type: &'a FuncType) -> Self {
        Self {
            name: Some(name),
            func_type,
        }
    }
}

impl<'a> From<&'a FuncType> for DisplayFuncType<'a> {
    fn from(func_type: &'a FuncType) -> Self {
        Self {
            name: None,
            func_type,
        }
    }
}

impl fmt::Display for DisplayFuncType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = self.name {
            f.write_str("fn ")?;
            name.fmt(f)?;
            f.write_str("(")?;
        } else {
            f.write_str("fn(")?;
        }
        let params = self.func_type.params();
        let results = self.func_type.results();
        DisplaySequence::new(", ", params.iter().map(DisplayValueType::from)).fmt(f)?;
        f.write_str(")")?;
        if !results.is_empty() {
            f.write_str(" -> ")?;
            if results.len() == 1 {
                DisplayValueType::from(&results[0]).fmt(f)?;
            } else {
                f.write_str("(")?;
                DisplaySequence::new(", ", results.iter().map(DisplayValueType::from)).fmt(f)?;
                f.write_str(")")?;
            }
        }
        Ok(())
    }
}

/// [`Display`]-wrapper for generic sequences with separators.
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
            first.fmt(f)?;
            for param in rest {
                f.write_str(separator)?;
                param.fmt(f)?;
            }
        }
        Ok(())
    }
}

/// [`Display`]-wrapper for exported functions of a [`Context`].
pub struct DisplayExportedFuncs<'a>(&'a Context);

impl<'a> From<&'a Context> for DisplayExportedFuncs<'a> {
    fn from(ctx: &'a Context) -> Self {
        Self(ctx)
    }
}

impl Display for DisplayExportedFuncs<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let exported_funcs = self.0.exported_funcs().collect::<Box<[_]>>();
        if exported_funcs.is_empty() {
            return f.write_str("No exported functions found for the Wasm module.");
        }
        f.write_str("The Wasm module exports the following functions:\n\n")?;
        for func in exported_funcs
            .iter()
            .map(|(name, func_type)| DisplayFuncType::new(name, func_type))
        {
            f.write_str(" - ")?;
            func.fmt(f)?;
        }
        Ok(())
    }
}
