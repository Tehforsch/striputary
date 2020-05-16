pub mod args;
pub mod audio_excerpt;
pub mod audio_time;
pub mod config;
pub mod cut;
pub mod dbus;
pub mod recorder;
pub mod recording_session;
pub mod song;
pub mod wav;
pub mod yaml_session;

use crate::args::parse_args;
use crate::config::{
    DEFAULT_BUFFER_FILE, DEFAULT_SESSION_FILE, POLLING_LOOP_TIMEOUT, TIME_AFTER_SESSION_END,
    TIME_BEFORE_SESSION_START, WAIT_TIME_BEFORE_FIRST_SONG,
};
use crate::dbus::{previous_song, start_playback, stop_playback};
use crate::recording_session::RecordingSession;
use log::info;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

fn main() -> Result<(), hound::Error> {
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
    cut::cut_session(session)
}

fn record_new_session(session_dir: PathBuf, buffer_file: PathBuf) -> RecordingSession {
    create_dir_all(&session_dir).expect("Failed to create simulation dir");
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

    // This ensures that spotify starts playing something, thus registering the
    // pulse audio sink.
    info!("Beginning pre-session phase");
    start_playback();
    thread::sleep(Duration::from_secs_f64(TIME_BEFORE_SESSION_START));
    stop_playback();
    info!("Going to beginning of song");
    previous_song();
    let mut playback_stopped = false;
    thread::sleep(Duration::from_secs_f64(WAIT_TIME_BEFORE_FIRST_SONG));
    info!("Starting playback.");
    start_playback();
    // We run until either interrupted via Ctrl+c or playback in spotify is stopped.
    // When playback was stopped we will assume the last song ran until completion.
    // However, when spotify reports that playback has been stopped it also reports
    while !playback_stopped && running.load(Ordering::SeqCst) {
        playback_stopped = dbus::collect_dbus_timestamps(record_start_time, &mut session);
    }
    if playback_stopped {
        info!("Playback was stopped. Waiting a few seconds to allow the recording to have a buffer at the end");
        thread::sleep(Duration::from_secs_f64(TIME_AFTER_SESSION_END));
    }
    return session;
}
