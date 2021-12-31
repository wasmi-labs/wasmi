#![allow(dead_code)] // TODO: remove

use anyhow::Result;
use std::fs;

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
}
