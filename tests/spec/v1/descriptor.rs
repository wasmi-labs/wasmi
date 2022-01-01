#![allow(dead_code)] // TODO: remove

use anyhow::Result;
use std::{
    fmt::{self, Display},
    fs,
};

/// The desciptor of a Wasm spec test suite run.
#[derive(Debug)]
pub struct TestDescriptor {
    /// The name of the Wasm spec test.
    name: String,
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
    pub fn new(name: &str) -> Result<Self> {
        let path = format!("tests/spec/testsuite/{}.wast", name);
        let file = fs::read_to_string(&path)?;
        let name = name.to_string();
        Ok(Self { name, path, file })
    }

    /// Returns the name of the Wasm spec test.
    pub fn name(&self) -> &str {
        &self.name
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
    pub fn spanned(&self, span: wast::Span) -> TestSpan {
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
    span: wast::Span,
}

impl Display for TestSpan<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (line, col) = self.span.linecol_in(self.contents);
        write!(f, "{}:{}:{}", self.path, line, col)
    }
}
