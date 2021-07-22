use clap::Clap;
use std::path::PathBuf;

#[derive(Clap, Debug)]
pub struct OffsetPosition {
    pub position: f64,
}

#[derive(Clap, Debug)]
pub enum OffsetOpts {
    Auto,
    Interactive,
    Manual(OffsetPosition),
    Graphical
}

#[derive(Clap, Debug)]
pub struct CutOpts {
    #[clap(subcommand)]
    pub offset: OffsetOpts,

    #[clap(
        short,
        long,
        about = "Instead of cutting songs grouped by album, group them by a given chunk size"
    )]
    pub chunk_size: Option<usize>,
}

#[derive(Clap, Debug)]
pub enum Action {
    Run(CutOpts),
    Record,
    Cut(CutOpts),
}

#[derive(Clap, Debug)]
pub struct Opts {
    pub session_dir: PathBuf,
    pub service_name: Option<String>,
    #[clap(subcommand)]
    pub action: Action,
}
