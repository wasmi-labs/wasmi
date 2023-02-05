use core::{fmt, fmt::Display};

/// Errors that can occur upon type checking function signatures.
#[derive(Debug)]
pub enum FuncError {
    /// The exported function could not be found.
    ExportedFuncNotFound,
    /// A function parameter did not match the required type.
    MismatchingParameterType,
    /// Specified an incorrect number of parameters.
    MismatchingParameterLen,
    /// A function result did not match the required type.
    MismatchingResultType,
    /// Specified an incorrect number of results.
    MismatchingResultLen,
}

impl Display for FuncError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FuncError::ExportedFuncNotFound => {
                write!(f, "could not find exported function")
            }
            FuncError::MismatchingParameterType => {
                write!(f, "encountered incorrect function parameter type")
            }
            FuncError::MismatchingParameterLen => {
                write!(f, "encountered an incorrect number of parameters")
            }
            FuncError::MismatchingResultType => {
                write!(f, "encountered incorrect function result type")
            }
            FuncError::MismatchingResultLen => {
                write!(f, "encountered an incorrect number of results")
            }
        }
    }
}
