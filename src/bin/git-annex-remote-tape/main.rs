#![allow(dead_code, unused_variables)]

use clap::Parser;
mod cli;
mod command;
mod error;
mod extension;
mod job;
mod remote;
mod tape;

use crate::cli::Command;
use crate::remote::Remote;

fn main() {
    let args = cli::Cli::parse();

    match args.command {
        Some(Command::Tape { command }) => tape::run(command),
        Some(Command::Jobs { command }) => job::run(command),
        None => Remote::new().run(),
    }
}
