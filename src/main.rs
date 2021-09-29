pub mod args;
pub mod audio_excerpt;
pub mod audio_time;
pub mod config;
pub mod cut;
pub mod dbus;
pub mod errors;
pub mod excerpt_collection;
pub mod excerpt_collections;
// pub mod gui;
pub mod gui;
pub mod path_utils;
pub mod record;
pub mod recorder;
pub mod recording_session;
pub mod service_config;
pub mod song;
pub mod wav;
pub mod yaml_session;

use crate::gui::TemplateApp;
use crate::{
    path_utils::{get_buffer_file, get_yaml_file},
    record::RecordingExitStatus,
    recording_session::RecordingSession,
};
use anyhow::{anyhow, Context, Result};
use args::Opts;
use clap::Clap;
use cut::get_excerpt_collection;
use excerpt_collections::ExcerptCollections;
use path_utils::get_yaml_files;
use record::record_new_session;
use service_config::ServiceConfig;
use std::{
    io::stdin,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

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
            let sessions =
                record_sessions_and_save_session_files(&args.session_dir, stream_config)?;
            wait_for_user_after_recording()?;
            run_gui(sessions);
        }
    };
    Ok(())
}

pub fn record_sessions_and_save_session_files(
    session_dir: &Path,
    stream_config: &ServiceConfig,
) -> Result<Vec<RecordingSession>> {
    let sessions = vec![];
    let is_running = set_ctrl_handler()?;
    for num in 0.. {
        let yaml_file = get_yaml_file(session_dir, num);
        let buffer_file = get_buffer_file(session_dir, num);
        let (status, session) =
            record_new_session(&yaml_file, &buffer_file, stream_config, is_running.clone())?;
        yaml_session::save(&session)?;
        if status == RecordingExitStatus::FinishedOrInterrupted {
            break;
        }
    }
    Ok(sessions)
}

fn set_ctrl_handler() -> Result<Arc<AtomicBool>> {
    let is_running = Arc::new(AtomicBool::new(true));
    let is_running_cloned = is_running.clone();
    ctrlc::set_handler(move || {
        is_running_cloned.store(false, Ordering::SeqCst);
    })
    .context("Error setting Ctrl-C handler")?;
    Ok(is_running)
}

fn load_sessions_and_cut(session_dir: &Path) -> Result<()> {
    let files = get_yaml_files(session_dir);
    if files.is_empty() {
        return Err(anyhow!("No session files found!"));
    }
    let sessions = files
        .iter()
        .map(|yaml_file| yaml_session::load(&yaml_file))
        .collect::<Result<Vec<_>>>();
    run_gui(sessions?);
    Ok(())
}

fn run_gui(sessions: Vec<RecordingSession>) {
    let collections = ExcerptCollections::new(sessions.into_iter().map(get_excerpt_collection).collect());
    let app = TemplateApp::new(collections);
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}

fn wait_for_user_after_recording() -> Result<()> {
    println!("Recording finished. Press enter to cut songs");
    let mut s = String::new();
    stdin().read_line(&mut s)?;
    Ok(())
}
