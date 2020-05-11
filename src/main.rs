pub mod config;
pub mod dbus;
pub mod recorder;
pub mod recording_session;
pub mod song;

use crate::recording_session::RecordingSession;
use std::path::Path;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use log::info;

fn main() {
    // Set up logging
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    // Start recording
    let session_dir = Path::new("./recordings");
    let buffer_file = session_dir.join(Path::new("buffer.ogg"));
    let recording_handles = recorder::record(&buffer_file);
    // Check for dbus signals while recording until terminated
    polling_loop(&session_dir);
    // When the user stopped the loop, kill the recording processes too
    recorder::stop_recording(recording_handles);
}

fn polling_loop(session_dir: &Path) {
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
}
