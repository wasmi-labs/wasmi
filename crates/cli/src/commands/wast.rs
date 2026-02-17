use crate::commands::Command;
use anyhow::{Context as _, Error};
use clap::Parser;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Executes a WebAssembly Script file.
#[derive(Parser)]
pub struct WastCommand {
    /// The file containing the WebAssembly Script (Wast) file.
    #[clap(
        required = true,
        value_name = "SCRIPT_FILE",
        value_hint = clap::ValueHint::FilePath,
    )]
    wast: PathBuf,
}

impl Command for WastCommand {
    fn execute(self) -> Result<(), Error> {
        let wast_file = self.wast_file();
        let wast = fs::read_to_string(self.wast_file())
            .with_context(|| format!("failed to read .wast file: {wast_file:?}"))?;
        let mut config = wasmi::Config::default();
        config.wasm_custom_page_sizes(true);
        config.wasm_wide_arithmetic(true);
        let mut runner = wasmi_wast::WastRunner::new(&config);
        runner
            .register_spectest()
            .context("failed to register spectest")?;
        runner
            .register_wasmitest()
            .context("failed to register wasmitest")?;
        runner.process_directives(wast_file.as_os_str().to_str().unwrap_or(""), &wast)
    }
}

impl WastCommand {
    /// Returns the Wast file path.
    fn wast_file(&self) -> &Path {
        self.wast.as_path()
    }
}
