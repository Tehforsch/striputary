use std::fs::create_dir_all;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread::{self};
use std::time::Duration;
use std::time::Instant;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;

use super::dbus::collect_dbus_info;
use super::dbus::previous_song;
use super::dbus::start_playback;
use super::dbus::stop_playback;
use super::recording_status::RecordingExitStatus;
use crate::config::TIME_AFTER_SESSION_END;
use crate::config::TIME_BEFORE_SESSION_START;
use crate::config::WAIT_TIME_BEFORE_FIRST_SONG;
use crate::recording::recorder;
use crate::recording::recording_status::RecordingStatus;
use crate::recording_session::RecordingSession;
use crate::run_args::RunArgs;
use crate::song::Song;

pub struct RecordingThread {
    run_args: RunArgs,
    is_running: Arc<AtomicBool>,
    song_sender: Sender<Song>,
}

impl RecordingThread {
    pub fn new(is_running: Arc<AtomicBool>, song_sender: Sender<Song>, run_args: &RunArgs) -> Self {
        Self {
            run_args: run_args.clone(),
            is_running,
            song_sender,
        }
    }

    pub fn record_new_session(&self) -> Result<(RecordingExitStatus, RecordingSession)> {
        create_dir_all(&self.run_args.session_dir).context("Failed to create session directory")?;
        if self.run_args.get_buffer_file().exists() {
            return Err(anyhow!(
                "Buffer file already exists, not recording a new session."
            ));
        }
        let recording_handles = recorder::start_recording(
            &self.run_args.get_buffer_file(),
            &self.run_args.service_config,
        )?;
        let record_start_time = Instant::now();
        let (status, session) =
            self.polling_loop(&self.run_args.get_yaml_file(), &record_start_time)?;
        recorder::stop_recording(recording_handles)?;
        session.save()?;
        self.is_running.store(false, Ordering::SeqCst);
        Ok((status, session))
    }

    fn polling_loop(
        &self,
        session_file: &Path,
        record_start_time: &Instant,
    ) -> Result<(RecordingExitStatus, RecordingSession)> {
        self.initial_buffer_phase()?;
        let (status, session) = self.recording_phase(session_file, record_start_time)?;
        self.final_buffer_phase();
        Ok((status, session))
    }

    fn initial_buffer_phase(&self) -> Result<()> {
        // Add a small time buffer before starting the playback properly.
        // This ensures that something starts playing, thus registering the
        // pulse audio sink. Also it avoids overflows when calculating the offset
        println!("Begin pre-session phase");
        start_playback(&self.run_args.service_config)?;
        thread::sleep(Duration::from_secs_f64(TIME_BEFORE_SESSION_START));
        stop_playback(&self.run_args.service_config)?;
        println!("Go to beginning of song");
        previous_song(&self.run_args.service_config)?;
        thread::sleep(Duration::from_secs_f64(WAIT_TIME_BEFORE_FIRST_SONG));
        Ok(())
    }

    fn recording_phase(
        &self,
        session_file: &Path,
        record_start_time: &Instant,
    ) -> Result<(RecordingExitStatus, RecordingSession)> {
        let recording_start_time = Instant::now()
            .duration_since(*record_start_time)
            .as_secs_f64();
        let mut session = RecordingSession::new(&session_file, recording_start_time);
        println!("Start playback.");
        start_playback(&self.run_args.service_config)?;
        loop {
            let num_songs_before = session.songs.len();
            let playback_status = collect_dbus_info(&mut session, &self.run_args.service_config)?;
            let num_songs_after = session.songs.len();
            if let RecordingStatus::Finished(exit_status) = playback_status {
                return Ok((exit_status, session));
            }
            // There should only be one new song if the delay between dbus signals is not too large, but you never know
            for song_index in num_songs_before - 1..num_songs_after - 1 {
                self.add_new_song(session.songs[song_index].clone());
            }
        }
    }

    fn add_new_song(&self, song: Song) {
        self.song_sender.send(song).unwrap();
    }

    fn final_buffer_phase(&self) {
        println!("Recording finished. Record final buffer for a few seconds");
        thread::sleep(Duration::from_secs_f64(TIME_AFTER_SESSION_END));
    }
}
