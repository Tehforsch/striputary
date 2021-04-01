pub mod args;
pub mod audio_excerpt;
pub mod audio_time;
pub mod config;
pub mod cut;
pub mod dbus;
pub mod errors;
pub mod record;
pub mod recorder;
pub mod recording_session;
pub mod service_config;
pub mod song;
pub mod wav;
pub mod yaml_session;

use crate::config::{DEFAULT_BUFFER_FILE, DEFAULT_SESSION_FILE};
use crate::recording_session::RecordingSession;
use anyhow::Result;
use args::{CutOpts, Opts};
use clap::Clap;
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
    let buffer_file = args.session_dir.join(DEFAULT_BUFFER_FILE);
    let yaml_file = args.session_dir.join(DEFAULT_SESSION_FILE);
    match &args.action {
        args::Action::Record => {
            record_session_and_save_session_file(
                &args.session_dir,
                &buffer_file,
                &yaml_file,
                stream_config,
            )?;
        }
        args::Action::Cut(cut_opts) => {
            load_session_and_cut_file(&yaml_file, &cut_opts)?;
        }
        args::Action::Run(cut_opts) => {
            let session = record_session_and_save_session_file(
                &args.session_dir,
                &buffer_file,
                &yaml_file,
                stream_config,
            )?;
            wait_for_user_after_recording()?;
            cut::cut_session(&session, cut_opts)?;
        }
    };
    Ok(())
}

pub fn record_session_and_save_session_file(
    session_dir: &Path,
    buffer_file: &Path,
    session_file: &Path,
    stream_config: &ServiceConfig,
) -> Result<RecordingSession> {
    let session = record_new_session(session_dir, buffer_file, stream_config)?;
    yaml_session::save(session_file, &session)?;
    Ok(session)
}

fn load_session_and_cut_file(yaml_file: &Path, cut_opts: &CutOpts) -> Result<()> {
    let mut session = yaml_session::load(&yaml_file)?;
    session.dir = yaml_file.parent().unwrap().into();
    cut::cut_session(&session, cut_opts)
}

fn wait_for_user_after_recording() -> Result<()> {
    println!("Recording finished. Press enter to cut songs");
    let mut s = String::new();
    stdin().read_line(&mut s)?;
    Ok(())
}
