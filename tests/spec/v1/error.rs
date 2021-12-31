use std::{error::Error, fmt, fmt::Display};

/// Errors that may occur upon Wasm spec test suite execution.
#[derive(Debug)]
pub enum TestError {
    InstanceNotRegistered { name: String },
    NoModuleInstancesFound,
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
        }
    }
}
