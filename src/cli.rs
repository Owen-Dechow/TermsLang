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

        #[arg(help = "Whether to send formats to stdout.")]
        to_stdout: Option<bool>,
    },

    #[command(about = "Update TermsLang.")]
    Update,

    #[command(about = "Get LSP info.")]
    Lsp {
        #[arg(help = "TermsLang File.")]
        file: PathBuf,

        #[arg(help = "Line to get variable info for.")]
        line: usize,

        #[arg(help = "Col to get variable info for.")]
        col: usize,

        #[arg(help = "Whether to run parsing.")]
        run_parse: Option<bool>,
    },
}
