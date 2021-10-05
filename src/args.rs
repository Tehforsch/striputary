use clap::Clap;
use std::path::PathBuf;

#[derive(Clap, Debug)]
pub struct Opts {
    pub output_dir: Option<PathBuf>,
    pub service_name: Option<String>,
}
