mod run;
mod wast;

use anyhow::Error;
use clap::Parser;

#[cfg(feature = "run")]
pub use self::run::RunCommand;
#[cfg(feature = "wast")]
pub use self::wast::WastCommand;

#[derive(Parser)]
#[command(
    name = "wasmi",
    version,
    about,
    after_help = "If a subcommand is not provided, the `run` subcommand will be used.",
)]
#[cfg_attr(
    feature = "run",
    // Note: this is required to enable the pattern that either a command is required
    //       or the run command is used by default.
    command(args_conflicts_with_subcommands = true),
)]
pub struct WasmiApp {
    #[cfg(not(feature = "run"))]
    #[command(subcommand)]
    subcommand: SubCommand,

    #[cfg(feature = "run")]
    #[command(subcommand)]
    subcommand: Option<SubCommand>,
    #[command(flatten)]
    #[cfg(feature = "run")]
    run: RunCommand,
}

#[derive(Parser)]
pub enum SubCommand {
    #[cfg(feature = "run")]
    Run(RunCommand),
    #[cfg(feature = "wast")]
    Wast(WastCommand),
}

/// Implemented by sub-commands in order to execute them.
pub trait Command {
    /// Executes the command.
    fn execute(self) -> Result<(), Error>;
}

impl Command for WasmiApp {
    fn execute(self) -> Result<(), Error> {
        #[cfg(feature = "run")]
        let subcommand = self.subcommand.unwrap_or(SubCommand::Run(self.run));
        #[cfg(not(feature = "run"))]
        let subcommand = self.subcommand;

        match subcommand {
            #[cfg(feature = "run")]
            SubCommand::Run(c) => c.execute(),

            #[cfg(feature = "wast")]
            SubCommand::Wast(c) => c.execute(),
        }
    }
}
