pub mod args;
pub mod audio_excerpt;
pub mod audio_time;
pub mod config;
pub mod cut;
pub mod dbus;
pub mod errors;
pub mod gui;
pub mod path_utils;
pub mod record;
pub mod recorder;
pub mod recording_session;
pub mod service_config;
pub mod song;
pub mod wav;
pub mod yaml_session;
pub mod excerpt_collection;
pub mod excerpt_collections;

use crate::{record::RecordingExitStatus, recording_session::RecordingSession};
use anyhow::{anyhow, Result};
use args::Opts;
use clap::Clap;
use path_utils::get_yaml_files;
use record::record_new_session;
use service_config::ServiceConfig;
use std::{io::stdin, path::Path};

fn main() -> Result<(), anyhow::Error> {
    let args = Opts::parse();
    let service_config = ServiceConfig::from_service_name(&get_service_name(&args.service_name))?;
    run_striputary(&args, &service_config)
}

fn get_service_name(service_name: &Option<String>) -> &str {
    service_name.as_deref().unwrap_or(config::DEFAULT_SERVICE)
}

fn run_striputary(args: &Opts, stream_config: &ServiceConfig) -> Result<()> {
    match &args.action {
        args::Action::Record => {
            record_sessions_and_save_session_files(&args.session_dir, stream_config)?;
        }
        args::Action::Cut => {
            load_sessions_and_cut(&args.session_dir)?;
        }
        args::Action::Run => {
            let sessions = record_sessions_and_save_session_files(&args.session_dir, stream_config)?;
            wait_for_user_after_recording()?;
            gui::run(sessions);
        }
    };
    Ok(())
}

pub fn record_sessions_and_save_session_files(
    session_dir: &Path,
    stream_config: &ServiceConfig,
) -> Result<Vec<RecordingSession>> {
    let sessions = vec![];
    loop {
        let (status, session) = record_new_session(session_dir, stream_config)?;
        yaml_session::save(&session)?;
        if status == RecordingExitStatus::FinishedOrInterrupted {
            break;
        }
    }
    Ok(sessions)
}

fn load_sessions_and_cut(session_dir: &Path) -> Result<()> {
    let files = get_yaml_files(session_dir);
    if files.len() == 0 {
        return Err(anyhow!("No session files found!"));
    }
    let sessions = files.iter().map(|yaml_file| yaml_session::load(&yaml_file)).collect::<Result<Vec<_>>>();
    gui::run(sessions?);
    Ok(())
}

fn wait_for_user_after_recording() -> Result<()> {
    println!("Recording finished. Press enter to cut songs");
    let mut s = String::new();
    stdin().read_line(&mut s)?;
    Ok(())
}
