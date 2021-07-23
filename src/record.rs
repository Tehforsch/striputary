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

#[derive(PartialEq)]
pub enum RecordingExitStatus {
    FinishedOrInterrupted,
    AlbumFinished,
}

#[derive(PartialEq)]
pub enum RecordingStatus {
    Running,
    Finished(RecordingExitStatus),
}

pub fn record_new_session(
    session_file: &Path,
    buffer_file: &Path,
    stream_config: &ServiceConfig,
    is_running: Arc<AtomicBool>,
) -> Result<(RecordingExitStatus, RecordingSession)> {
    create_dir_all(&session_file.parent().unwrap())
        .context("Failed to create session directory")?;
    if buffer_file.exists() {
        return Err(anyhow!(
            "Buffer file already exists, not recording a new session."
        ));
    }
    let recording_handles = recorder::start_recording(&buffer_file, stream_config)?;
    let record_start_time = Instant::now();
    // Check for dbus signals while recording until terminated
    let (status, session) =
        polling_loop(&record_start_time, &session_file, stream_config, is_running)?;
    // Whether the loop ended because of the user interrupt (ctrl-c) or
    // because the playback was stopped doesn't matter - kill the recording processes
    recorder::stop_recording(recording_handles)?;
    Ok((status, session))
}

fn polling_loop(
    record_start_time: &Instant,
    session_filename: &Path,
    stream_config: &ServiceConfig,
    is_running: Arc<AtomicBool>,
) -> Result<(RecordingExitStatus, RecordingSession)> {
    initial_buffer_phase(stream_config)?;
    let (status, session) = recording_phase(
        session_filename,
        record_start_time,
        is_running,
        stream_config,
    )?;
    final_buffer_phase();
    Ok((status, session))
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
    session_filename: &Path,
    record_start_time: &Instant,
    is_running: Arc<AtomicBool>,
    stream_config: &ServiceConfig,
) -> Result<(RecordingExitStatus, RecordingSession)> {
    let recording_start_time = Instant::now()
        .duration_since(*record_start_time)
        .as_secs_f64();
    let mut session = RecordingSession::new(session_filename, recording_start_time);
    println!("Start playback.");
    start_playback(stream_config)?;
    loop {
        let playback_status = collect_dbus_info(&mut session, stream_config)?;
        if let RecordingStatus::Finished(exit_status) = playback_status {
            return Ok((exit_status, session));
        }
        if !is_running.load(Ordering::SeqCst) {
            return Ok((RecordingExitStatus::FinishedOrInterrupted, session));
        }
    }
}

fn final_buffer_phase() {
    println!("Recording finished. Record final buffer for a few seconds");
    thread::sleep(Duration::from_secs_f64(TIME_AFTER_SESSION_END));
}
