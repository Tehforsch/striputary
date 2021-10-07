use std::path::PathBuf;

use clap::Clap;

#[derive(Clap, Debug)]
pub struct Opts {
    pub output_dir: Option<PathBuf>,
    pub service_name: Option<String>,
}
