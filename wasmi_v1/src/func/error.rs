use super::Func;
use core::{fmt, fmt::Display};

/// Errors that can occur upon operating with [`Func`] instances.
#[derive(Debug)]
pub enum FuncError {
    /// Encountered when trying to create a [`TypedFunc`]
    /// with mismatching function parameter types.
    ///
    /// [`TypedFunc`]: [`super::TypedFunc`]
    MismatchingParameters { func: Func },
    /// Encountered when trying to create a [`TypedFunc`]
    /// with mismatching function results types.
    ///
    /// [`TypedFunc`]: [`super::TypedFunc`]
    MismatchingResults { func: Func },
}

impl Display for FuncError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FuncError::MismatchingParameters { func } => write!(
                f,
                "encountered mismatching function parameter types for TypedFunc: {:?}",
                func
            ),
            FuncError::MismatchingResults { func } => write!(
                f,
                "encountered mismatching function result types for TypedFunc: {:?}",
                func
            ),
        }
    }
}
