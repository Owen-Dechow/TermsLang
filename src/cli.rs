use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
pub struct Args {
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Subcommand)]
pub enum Command {
    // Run a program
    Run {
        // File containing entry function.
        file: PathBuf,

        // Command line arguments
        args: Vec<String>,
    },

    // Run a program
    Debug {
        // File containing entry function.
        file: PathBuf,

        // Command line arguments
        args: Vec<String>,
    },

    // Format a file
    Format {
        // File with valid syntax
        file: PathBuf,
    },

    // Update TermsLang
    Update,
}
