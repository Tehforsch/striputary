use crate::dbus::{collect_dbus_info, previous_song, start_playback, stop_playback};
use crate::recorder;
use crate::recording_session::RecordingSession;
use crate::{
    config::{TIME_AFTER_SESSION_END, TIME_BEFORE_SESSION_START, WAIT_TIME_BEFORE_FIRST_SONG},
    service_config::ServiceConfig,
};
use anyhow::{anyhow, Context, Result};
use std::fs::create_dir_all;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

pub fn record_new_session(
    session_dir: &Path,
    buffer_file: &Path,
    stream_config: &ServiceConfig,
) -> Result<RecordingSession> {
    create_dir_all(&session_dir).context("Failed to create session directory")?;
    if buffer_file.exists() {
        return Err(anyhow!(
            "Buffer file already exists, not recording a new session."
        ));
    }
    let recording_handles = recorder::start_recording(&buffer_file, stream_config)?;
    let record_start_time = Instant::now();
    // Check for dbus signals while recording until terminated
    let session = polling_loop(&record_start_time, &session_dir, stream_config)?;
    // Whether the loop ended because of the user interrupt (ctrl-c) or
    // because the playback was stopped doesn't matter - kill the recording processes
    recorder::stop_recording(recording_handles)?;
    Ok(session)
}

fn polling_loop(
    record_start_time: &Instant,
    session_dir: &Path,
    stream_config: &ServiceConfig,
) -> Result<RecordingSession> {
    let is_running = Arc::new(AtomicBool::new(true));

    let is_running_clone = is_running.clone();
    set_ctrl_handler(is_running_clone)?;

    initial_buffer_phase(stream_config)?;
    let session = recording_phase(session_dir, record_start_time, is_running, stream_config)?;
    final_buffer_phase();
    Ok(session)
}

fn initial_buffer_phase(stream_config: &ServiceConfig) -> Result<()> {
    // Add a small time buffer before starting the playback properly.
    // This ensures that something starts playing, thus registering the
    // pulse audio sink. Also it avoids overflows when calculating the offset
    println!("Begin pre-session phase");
    start_playback(stream_config)?;
    thread::sleep(Duration::from_secs_f64(TIME_BEFORE_SESSION_START));
    stop_playback(stream_config)?;
    println!("Go to beginning of song");
    previous_song(stream_config)?;
    thread::sleep(Duration::from_secs_f64(WAIT_TIME_BEFORE_FIRST_SONG));
    Ok(())
}

fn recording_phase(
    session_dir: &Path,
    record_start_time: &Instant,
    is_running: Arc<AtomicBool>,
    stream_config: &ServiceConfig,
) -> Result<RecordingSession> {
    let recording_start_time = Instant::now()
        .duration_since(*record_start_time)
        .as_secs_f64();
    let mut session = RecordingSession::new(session_dir, recording_start_time);
    let mut playback_stopped = false;
    println!("Start playback.");
    start_playback(stream_config)?;
    while !playback_stopped && is_running.load(Ordering::SeqCst) {
        playback_stopped = collect_dbus_info(&mut session, stream_config);
    }
    Ok(session)
}

fn final_buffer_phase() {
    println!("Recording finished. Record final buffer for a few seconds");
    thread::sleep(Duration::from_secs_f64(TIME_AFTER_SESSION_END));
}

fn set_ctrl_handler(is_running: Arc<AtomicBool>) -> Result<()> {
    ctrlc::set_handler(move || {
        is_running.store(false, Ordering::SeqCst);
    })
    .context("Error setting Ctrl-C handler")
}
