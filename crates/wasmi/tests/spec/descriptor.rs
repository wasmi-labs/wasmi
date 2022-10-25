use std::{
    fmt::{self, Display},
    fs,
};
use wast::token::Span;

/// The desciptor of a Wasm spec test suite run.
#[derive(Debug)]
pub struct TestDescriptor {
    /// The path of the Wasm spec test `.wast` file.
    path: String,
    /// The contents of the Wasm spec test `.wast` file.
    file: String,
}

impl TestDescriptor {
    /// Creates a new Wasm spec [`TestDescriptor`].
    ///
    /// # Errors
    ///
    /// If the corresponding Wasm test spec file cannot properly be read.
    pub fn new(name: &str) -> Self {
        let path = format!("tests/spec/{name}.wast");
        let file = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{path}, failed to read `.wast` test file: {error}"));
        Self { path, file }
    }

    /// Returns the path of the Wasm spec test `.wast` file.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns the contents of the Wasm spec test `.wast` file.
    pub fn file(&self) -> &str {
        &self.file
    }

    /// Creates a [`TestSpan`] which can be used to print the location within the `.wast` test file.
    pub fn spanned(&self, span: Span) -> TestSpan {
        TestSpan {
            path: self.path(),
            contents: self.file(),
            span,
        }
    }
}

/// Useful for printing the location where the `.wast` parse is located.
#[derive(Debug)]
pub struct TestSpan<'a> {
    /// The file path of the `.wast` test.
    path: &'a str,
    /// The file contents of the `.wast` test.
    contents: &'a str,
    /// The line and column within the `.wast` test file.
    span: Span,
}

impl Display for TestSpan<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (line, col) = self.span.linecol_in(self.contents);
        write!(f, "{}:{line}:{col}", self.path)
    }
}
