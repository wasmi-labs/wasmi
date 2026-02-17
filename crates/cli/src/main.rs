#![cfg_attr(
    // Note: if all sub-commands are disabled we silence dead_code warnings
    not(any(
        feature = "run",
        feature = "wast",
    )),
    allow(dead_code),
)]

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
