use clap::Clap;
use std::path::PathBuf;

#[derive(Clap, Debug)]
pub struct OffsetPosition {
    pub position: f64,
}

#[derive(Clap, Debug)]
pub enum Action {
    Run,
    Record,
    Cut,
}

#[derive(Clap, Debug)]
pub struct Opts {
    pub session_dir: PathBuf,
    pub service_name: Option<String>,
    #[clap(subcommand)]
    pub action: Action,
}
