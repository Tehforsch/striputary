pub mod args;
pub mod audio_excerpt;
pub mod audio_time;
pub mod config;
pub mod cut;
pub mod errors;
pub mod excerpt_collection;
pub mod gui;
pub mod recording_session;
pub mod run_args;
pub mod service_config;
pub mod song;
pub mod wav;
pub mod yaml_session;
pub mod recording;

use crate::gui::StriputaryGui;
use crate::recording_session::RecordingSession;
use anyhow::{anyhow, Result};
use args::Opts;
use clap::Clap;
use cut::get_excerpt_collection;
use run_args::RunArgs;
use service_config::ServiceConfig;

fn main() -> Result<(), anyhow::Error> {
    let args = Opts::parse();
    let service_config = ServiceConfig::from_service_name(&get_service_name(&args.service_name))?;
    run_striputary(&args, service_config)
}

fn get_service_name(service_name: &Option<String>) -> &str {
    service_name.as_deref().unwrap_or(config::DEFAULT_SERVICE)
}

fn run_striputary(args: &Opts, stream_config: ServiceConfig) -> Result<()> {
    let run_args = RunArgs::new(&args.session_dir, stream_config);
    match &args.action {
        args::Action::Record => {
            run_gui_record(&run_args);
        }
        args::Action::Cut => {
            load_sessions_and_cut(&run_args)?;
        }
    };
    Ok(())
}

fn load_sessions_and_cut(run_args: &RunArgs) -> Result<()> {
    let files = run_args.get_yaml_files();
    if files.is_empty() {
        return Err(anyhow!("No session files found!"));
    }
    let sessions = files
        .iter()
        .map(|yaml_file| yaml_session::load(&yaml_file))
        .collect::<Result<Vec<_>>>();
    run_gui_cut(run_args, sessions?);
    Ok(())
}

fn run_gui_record(run_args: &RunArgs) {
    let app = StriputaryGui::new(run_args, vec![]);
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}

fn run_gui_cut(run_args: &RunArgs, sessions: Vec<RecordingSession>) {
    let collections = sessions.into_iter().map(get_excerpt_collection).collect();
    let app = StriputaryGui::new(run_args, collections);
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
