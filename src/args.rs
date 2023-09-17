use std::path::PathBuf;

use crate::service_config::Service;

#[derive(clap::StructOpt)]
#[clap(version)]
pub struct Opts {
    pub output_dir: Option<PathBuf>,
    pub service: Option<Service>,
    #[clap(short, parse(from_occurrences))]
    pub verbosity: usize,
}
