use clap::Parser;

#[derive(Parser)]
pub struct Args {
    #[arg()]
    pub file: String,

    #[arg(long, short)]
    pub format: bool,
}
