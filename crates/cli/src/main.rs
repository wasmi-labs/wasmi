use crate::commands::{Command, WasmiApp};
use anyhow::Result;
use clap::Parser;

mod commands;
mod context;
mod display;
mod utils;

#[cfg(test)]
mod tests;

fn main() -> Result<()> {
    WasmiApp::parse().execute()
}
