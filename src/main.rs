pub mod args;
pub mod audio_excerpt;
pub mod audio_time;
pub mod config;
pub mod config_file;
pub mod cut;
pub mod data_stream;
pub mod errors;
pub mod excerpt_collection;
pub mod gui;
pub mod recording;
pub mod recording_session;
pub mod run_args;
pub mod service_config;
pub mod song;
pub mod wav;
pub mod yaml_session;

use std::path::Path;

use crate::gui::StriputaryGui;
use anyhow::Result;
use args::Opts;
use clap::Clap;
use config_file::ConfigFile;
use service_config::ServiceConfig;

fn main() -> Result<(), anyhow::Error> {
    let args = Opts::parse();
    let config_file = ConfigFile::read();
    let output_dir = args
        .output_dir
        .or(config_file.ok().map(|file| file.output_dir));
    let service_config = ServiceConfig::from_service_name(&get_service_name(&args.service_name))?;
    match output_dir {
        Some(dir) => {
            Ok(run_gui(&dir, service_config))
        }
        None => panic!("Need an output folder - either pass it as a command line argument or specify it in the config file (probably ~/.config/striputary/config.yaml")
    }
}

fn get_service_name(service_name: &Option<String>) -> &str {
    service_name.as_deref().unwrap_or(config::DEFAULT_SERVICE)
}

fn run_gui(dir: &Path, service_config: ServiceConfig) {
    let app = StriputaryGui::new(dir, service_config);
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
