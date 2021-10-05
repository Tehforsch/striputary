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

use std::path::{Path, PathBuf};

use crate::gui::StriputaryGui;
use crate::recording_session::RecordingSession;
use anyhow::{anyhow, Result};
use args::Opts;
use chrono::Local;
use clap::Clap;
use config_file::ConfigFile;
use run_args::RunArgs;
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
            run_striputary(&dir, service_config, args.record)
        }
        None => panic!("Need an output folder - either pass it as a command line argument or specify it in the config file (probably ~/.config/striputary/config.yaml")
    }
}

fn get_service_name(service_name: &Option<String>) -> &str {
    service_name.as_deref().unwrap_or(config::DEFAULT_SERVICE)
}

fn run_striputary(output_dir: &Path, stream_config: ServiceConfig, record: bool) -> Result<()> {
    let run_args = RunArgs::new(&get_session_dir(output_dir), stream_config);
    let sessions = match record {
        true => vec![],
        false => load_sessions(&run_args)?,
    };
    run_gui(&run_args, sessions);
    Ok(())
}

fn get_session_dir(output_dir: &Path) -> PathBuf {
    let date_string = Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
    output_dir.join(&date_string)
}

fn load_sessions(run_args: &RunArgs) -> Result<Vec<RecordingSession>> {
    let files = run_args.get_yaml_files();
    if files.is_empty() {
        return Err(anyhow!("No session files found!"));
    }
    files
        .iter()
        .map(|yaml_file| yaml_session::load(&yaml_file))
        .collect::<Result<Vec<_>>>()
}

fn run_gui(run_args: &RunArgs, sessions: Vec<RecordingSession>) {
    let app = StriputaryGui::new(run_args, sessions);
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
