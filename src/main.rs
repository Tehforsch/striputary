pub mod args;
pub mod config;
pub mod cut;
pub mod dbus;
pub mod recorder;
pub mod recording_session;
pub mod song;
pub mod yaml_session;

use crate::args::parse_args;
use crate::config::{DEFAULT_BUFFER_FILE, DEFAULT_SESSION_FILE};
use crate::recording_session::RecordingSession;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

fn main() {
    let args = parse_args();
    let session_dir = args.session_dir;
    let yaml_file = session_dir.join(DEFAULT_SESSION_FILE);
    let buffer_file = session_dir.join(DEFAULT_BUFFER_FILE);

    // Set up logging
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    let session = match args.action {
        args::Action::Record => {
            let session = record_new_session(session_dir, buffer_file);
            yaml_session::save(yaml_file.as_path(), &session);
            session
        }
        args::Action::Load => yaml_session::load(yaml_file.as_path()),
    };
    cut::cut_session(session);
}

fn record_new_session(session_dir: PathBuf, buffer_file: PathBuf) -> RecordingSession {
    // Start recording
    let recording_handles = recorder::record(&buffer_file);
    let record_start_time = Instant::now();
    // Check for dbus signals while recording until terminated
    let session = polling_loop(&record_start_time, &session_dir);
    // When the user stopped the loop, kill the recording processes too
    recorder::stop_recording(recording_handles);
    session
}

fn polling_loop(record_start_time: &Instant, session_dir: &Path) -> RecordingSession {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    let mut session = RecordingSession {
        dir: session_dir.to_path_buf(),
        timestamps: vec![],
        songs: vec![],
    };

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    while running.load(Ordering::SeqCst) {
        dbus::collect_dbus_timestamps(record_start_time, &mut session);
    }
    return session;
}
