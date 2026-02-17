use crate::commands::Command;
use anyhow::{Context as _, Error};
use clap::Parser;
use std::{fs, path::PathBuf};

/// Executes a WebAssembly Script file.
#[derive(Parser)]
pub struct WastCommand {
    /// The file containing the WebAssembly Script (Wast) file.
    #[clap(
        required = true,
        value_name = "SCRIPT_FILE",
        value_hint = clap::ValueHint::FilePath,
    )]
    scripts: Vec<PathBuf>,
}

impl Command for WastCommand {
    fn execute(self) -> Result<(), Error> {
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
        for script in self.scripts() {
            let wast = fs::read_to_string(script)
                .with_context(|| format!("failed to read .wast file: {script:?}"))?;
            let file_name = script.as_os_str().to_str().unwrap_or("");
            runner.process_directives(file_name, &wast)?;
        }
        Ok(())
    }
}

impl WastCommand {
    /// Returns the Wast file path.
    fn scripts(&self) -> &[PathBuf] {
        &self.scripts[..]
    }
}
