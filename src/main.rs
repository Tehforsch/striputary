pub mod args;
pub mod config;
pub mod cut;
pub mod dbus;
pub mod recorder;
pub mod recording_session;
pub mod song;
pub mod yaml_session;

use crate::args::parse_args;
use crate::recording_session::RecordingSession;
use std::path::Path;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::config::{DEFAULT_BUFFER_FILE, DEFAULT_SESSION_FILE};

fn main() {
    let args = parse_args();
    let session_dir = args.session_dir;
    let yaml_file = session_dir.join(DEFAULT_SESSION_FILE);
    let buffer_file = session_dir.join(DEFAULT_BUFFER_FILE);

    // Set up logging
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    match args.action {
        args::Action::Record => {
            // Start recording
            let recording_handles = recorder::record(&buffer_file);
            // Check for dbus signals while recording until terminated
            let session = polling_loop(&session_dir);
            // When the user stopped the loop, kill the recording processes too
            recorder::stop_recording(recording_handles);
            yaml_session::save(yaml_file.as_path(), session)
        }
        args::Action::Load => {
            // println!("Loading ... this isnt actually implemented yet tho hello!");
            let session = yaml_session::load(yaml_file.as_path());
        }
    }
    // cut::cut_session(session);
}

fn polling_loop(session_dir: &Path) -> RecordingSession {
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
        dbus::collect_dbus_timestamps(&mut session);
    }
    return session;
}
