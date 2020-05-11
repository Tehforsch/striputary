pub mod args;
pub mod config;
pub mod cut;
pub mod dbus;
pub mod recorder;
pub mod recording_session;
pub mod song;

use crate::args::parse_args;
use crate::recording_session::RecordingSession;
use std::path::Path;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() {
    let args = parse_args();
    // Set up logging
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    if args.is_present("record") {
        // Start recording
        let session_dir = Path::new("./recordings");
        let buffer_file = session_dir.join(Path::new("buffer.ogg"));
        let recording_handles = recorder::record(&buffer_file);
        // Check for dbus signals while recording until terminated
        let session = polling_loop(&session_dir);
        // When the user stopped the loop, kill the recording processes too
        recorder::stop_recording(recording_handles);
    } else if args.is_present("load") {
        println!("Loading ... this isnt actually implemented yet tho hello!");
        // let session = load_session::load(session);
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
