use clap::Clap;
use std::path::PathBuf;

#[derive(Clap)]
pub struct Opts {
    #[clap(subcommand)]
    pub action: Action,
    pub session_dir: PathBuf,
    // #[clap(short, long, parse(from_occurrences))]
    // pub verbose: i32,
}

#[derive(Clap)]
pub enum Action {
    Record,
    Load,
}

pub fn parse_args() -> Opts {
    let opts: Opts = Opts::parse();
    opts
}
