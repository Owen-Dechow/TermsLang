use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Args {
    #[arg()]
    pub file: PathBuf,

    #[arg(long, short)]
    pub format: bool,
}
