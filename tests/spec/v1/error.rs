use std::{error::Error, fmt, fmt::Display};

use wasmi::Trap;

/// Errors that may occur upon Wasm spec test suite execution.
#[derive(Debug)]
pub enum TestError {
    Trap(Trap),
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
                write!(f, "missing module instance with name: {}", name)
            }
            Self::NoModuleInstancesFound => {
                write!(f, "found no module instances registered so far")
            }
            Self::FuncNotFound {
                module_name,
                func_name,
            } => {
                write!(
                    f,
                    "missing func exported as: {:?}::{}",
                    module_name, func_name
                )
            }
            Self::GlobalNotFound {
                module_name,
                global_name,
            } => {
                write!(
                    f,
                    "missing global variable exported as: {:?}::{}",
                    module_name, global_name
                )
            }
            Self::Trap(trap) => Display::fmt(trap, f),
        }
    }
}

impl From<Trap> for TestError {
    fn from(error: Trap) -> Self {
        Self::Trap(error)
    }
}
