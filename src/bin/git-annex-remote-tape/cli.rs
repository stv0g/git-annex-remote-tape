use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    Tape {
        #[command(subcommand)]
        command: Option<TapeCommand>,
    },

    Jobs {
        #[command(subcommand)]
        command: Option<JobCommand>,
    },
}

#[derive(Subcommand)]
pub enum TapeCommand {
    /// Intitialize a tape cartridge for use with git-annex-remote-tape.
    Init {},

    /// Erase all data from a tape cartridge.
    Erase {
        /// Perform a secure earse operation.
        #[arg(short, long)]
        secure: bool,
    },

    /// Get information about the tape cartridge.
    Info {},
}

#[derive(Subcommand)]
pub enum JobCommand {
    /// Show details about a pending retrieval job.
    Info { job_id: u32 },

    /// List pending retrieval jobs.
    List {},

    /// Start a single or all pending retrieval jobs.
    Start { job_id: Option<u32> },

    /// Drop single or all pendings retrieval jobs.
    Drop { job_id: Option<u32> },
}
