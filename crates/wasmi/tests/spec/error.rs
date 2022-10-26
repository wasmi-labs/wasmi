use std::{error::Error, fmt, fmt::Display};
use wasmi::Error as WasmiError;

/// Errors that may occur upon Wasm spec test suite execution.
#[derive(Debug)]
pub enum TestError {
    Wasmi(WasmiError),
    InstanceNotRegistered {
        name: String,
    },
    NoModuleInstancesFound,
    FuncNotFound {
        module_name: Option<String>,
        func_name: String,
    },
    GlobalNotFound {
        module_name: Option<String>,
        global_name: String,
    },
}

impl Error for TestError {}

impl Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InstanceNotRegistered { name } => {
                write!(f, "missing module instance with name: {name}")
            }
            Self::NoModuleInstancesFound => {
                write!(f, "found no module instances registered so far")
            }
            Self::FuncNotFound {
                module_name,
                func_name,
            } => {
                write!(f, "missing func exported as: {module_name:?}::{func_name}",)
            }
            Self::GlobalNotFound {
                module_name,
                global_name,
            } => {
                write!(
                    f,
                    "missing global variable exported as: {module_name:?}::{global_name}",
                )
            }
            Self::Wasmi(wasmi_error) => Display::fmt(wasmi_error, f),
        }
    }
}

impl<E> From<E> for TestError
where
    E: Into<WasmiError>,
{
    fn from(error: E) -> Self {
        Self::Wasmi(error.into())
    }
}
