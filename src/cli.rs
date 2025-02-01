use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    version,
    about = "The Terms Programming Language: https://github.com/Owen-Dechow/TermsLang",
    author = "Owen Dechow"
)]
pub struct Args {
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(about = "Run a program.")]
    Run {
        #[arg(help = "File containing entry function.")]
        file: PathBuf,

        #[arg(help = "Command line arguments.")]
        args: Vec<String>,
    },

    #[command(about = "Run a program.")]
    Debug {
        #[arg(help = "File containing entry function.")]
        file: PathBuf,

        #[arg(help = "Command line arguments")]
        args: Vec<String>,
    },

    #[command(about = "Format a file.")]
    Format {
        #[arg(help = "File with valid syntax")]
        file: PathBuf,
    },

    #[command(about = "Update TermsLang.")]
    Update,
}
