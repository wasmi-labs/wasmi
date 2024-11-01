use std::fmt::{self, Display};
use wast::token::Span;

/// The desciptor of a Wasm spec test suite run.
#[derive(Debug)]
pub struct TestDescriptor<'a> {
    /// The contents of the Wasm spec test `.wast` file.
    wast: &'a str,
}

impl<'a> TestDescriptor<'a> {
    /// Creates a new Wasm spec [`TestDescriptor`].
    ///
    /// # Errors
    ///
    /// If the corresponding Wasm test spec file cannot properly be read.
    pub fn new(wast: &'a str) -> Self {
        Self { wast }
    }

    /// Creates a [`ErrorPos`] which can be used to print the location within the `.wast` test file.
    pub fn spanned(&self, span: Span) -> ErrorPos<'a> {
        ErrorPos::new(self.wast, span)
    }
}

/// Useful for printing the location where the `.wast` parse is located.
#[derive(Debug)]
pub struct ErrorPos<'a> {
    /// The file contents of the `.wast` test.
    wast: &'a str,
    /// The line and column within the `.wast` test file.
    span: Span,
}

impl<'a> ErrorPos<'a> {
    /// Creates a new [`ErrorPos`].
    pub fn new(wast: &'a str, span: Span) -> Self {
        Self { wast, span }
    }
}

impl Display for ErrorPos<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (line, col) = self.span.linecol_in(self.wast);
        // Change from 0-indexing to 1-indexing for better UX:
        let line = line + 1;
        let col = col + 1;
        write!(f, "{line}:{col}")
    }
}
