use clap::arg_enum;
use std::path::PathBuf;
use structopt::StructOpt;

arg_enum! {
    #[derive(Debug)]
    pub enum OffsetOpts {
        Auto,
        Interactive,
    }
}

#[derive(StructOpt, Debug)]
pub struct Opts {
    #[structopt(subcommand)]
    pub action: Action,
    pub session_dir: PathBuf,
    // #[clap(short, long, parse(from_occurrences))]
    // pub verbose: i32,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "Action", about = "What to do")]
pub enum Action {
    #[structopt(name = "record")]
    Record,
    #[structopt()]
    Cut { offset: OffsetOpts },
}

pub fn parse_args() -> Opts {
    let opts: Opts = Opts::from_args();
    opts
}
