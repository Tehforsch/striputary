use clap::Clap;
use std::path::PathBuf;

#[derive(Clap)]
pub struct Opts {
    #[clap(subcommand)]
    pub action: Action,
    pub session_dir: PathBuf,
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: i32,
}

#[derive(Clap)]
pub enum Action {
    Record,
    Load,
}

pub fn parse_args() -> Opts {
    let opts: Opts = Opts::parse();

    match opts.verbose {
        0 => println!("No verbose info"),
        1 => println!("Some verbose info"),
        2 => println!("Tons of verbose info"),
        3 | _ => println!("Don't be crazy"),
    }
    opts
}
