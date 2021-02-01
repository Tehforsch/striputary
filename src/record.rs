use crate::config::{
    TIME_AFTER_SESSION_END, TIME_BEFORE_SESSION_START, WAIT_TIME_BEFORE_FIRST_SONG,
};
use crate::dbus::{collect_dbus_timestamps, previous_song, start_playback, stop_playback};
use crate::recorder;
use crate::recording_session::RecordingSession;
use anyhow::{anyhow, Context, Result};
use std::fs::create_dir_all;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

pub fn record_new_session(session_dir: &Path, buffer_file: &Path) -> Result<RecordingSession> {
    create_dir_all(&session_dir).expect("Failed to create simulation dir");
    if buffer_file.exists() {
        return Err(anyhow!(
            "Buffer file already exists, not recording a new session."
        ));
    }
    // Start recording
    let recording_handles = recorder::start_recording(&buffer_file);
    let record_start_time = Instant::now();
    // Check for dbus signals while recording until terminated
    let session = polling_loop(&record_start_time, &session_dir)?;
    // When the user stopped the loop, kill the recording processes too
    recorder::stop_recording(recording_handles);
    Ok(session)
}

fn polling_loop(record_start_time: &Instant, session_dir: &Path) -> Result<RecordingSession> {
    let mut session = RecordingSession::new(session_dir);

    let is_running = Arc::new(AtomicBool::new(true));

    let is_running_clone = is_running.clone();
    set_ctrl_handler(is_running_clone)?;

    initial_buffer_phase();
    record_phase(&mut session, record_start_time, is_running);
    final_buffer_phase();
    Ok(session)
}

fn initial_buffer_phase() -> () {
    // Add a small time buffer before starting the playback properly.
    // This ensures that spotify starts playing something, thus registering the
    // pulse audio sink.
    println!("Begin pre-session phase");
    start_playback();
    thread::sleep(Duration::from_secs_f64(TIME_BEFORE_SESSION_START));
    stop_playback();
    println!("Go to beginning of song");
    previous_song();
    thread::sleep(Duration::from_secs_f64(WAIT_TIME_BEFORE_FIRST_SONG));
}

fn record_phase(
    session: &mut RecordingSession,
    record_start_time: &Instant,
    is_running: Arc<AtomicBool>,
) {
    let mut playback_stopped = false;
    println!("Start playback.");
    start_playback();
    while !playback_stopped && is_running.load(Ordering::SeqCst) {
        playback_stopped = collect_dbus_timestamps(record_start_time, session);
    }
}

fn final_buffer_phase() {
    println!(
        "Recording finished. Wait a few seconds to allow the recording to have a buffer at the end"
    );
    thread::sleep(Duration::from_secs_f64(TIME_AFTER_SESSION_END));
}

fn set_ctrl_handler(is_running: Arc<AtomicBool>) -> Result<()> {
    ctrlc::set_handler(move || {
        is_running.store(false, Ordering::SeqCst);
    })
    .context("Error setting Ctrl-C handler")
}
