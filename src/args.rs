use std::path::PathBuf;

#[derive(clap::StructOpt)]
#[clap(version)]
pub struct Opts {
    pub output_dir: Option<PathBuf>,
    pub service_name: Option<String>,
}
