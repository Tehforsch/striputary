use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread::{self};
use std::time::Instant;

use anyhow::Result;
use log::debug;
use log::info;

use super::dbus::collect_dbus_info;
use super::dbus::previous_song;
use super::dbus::start_playback;
use super::dbus::stop_playback;
use super::recording_status::RecordingExitStatus;
use super::Recorder;
use crate::config;
use crate::config::TIME_AFTER_SESSION_END;
use crate::config::TIME_BEFORE_SESSION_START;
use crate::config::TIME_BETWEEN_SUBSEQUENT_DBUS_COMMANDS;
use crate::config::WAIT_TIME_BEFORE_FIRST_SONG;
use crate::recording::dbus::next_song;
use crate::recording::recording_status::RecordingStatus;
use crate::recording_session::RecordingSession;
use crate::song::Song;
use crate::Opts;

pub struct RecordingThread {
    opts: Opts,
    is_running: Arc<AtomicBool>,
    song_sender: Sender<Song>,
    session: RecordingSession,
    recorder: Recorder,
}

impl RecordingThread {
    pub fn new(is_running: Arc<AtomicBool>, song_sender: Sender<Song>, opts: &Opts) -> Self {
        let session = RecordingSession::new(&opts.get_yaml_file());
        let recorder = Recorder::start(&opts).unwrap();
        let recorder = Self {
            opts: opts.clone(),
            is_running,
            song_sender,
            session,
            recorder,
        };
        recorder
    }

    pub fn record_new_session(mut self) -> Result<(RecordingExitStatus, RecordingSession)> {
        let status = self.polling_loop()?;
        self.stop()?;
        Ok((status, self.session))
    }

    fn stop(&mut self) -> Result<()> {
        self.is_running.store(false, Ordering::SeqCst);
        self.recorder.stop()?;
        self.session.save()
    }

    fn polling_loop(&mut self) -> Result<RecordingExitStatus> {
        self.initial_buffer_phase()?;
        let status = self.recording_phase()?;
        self.final_buffer_phase();
        Ok(status)
    }

    fn initial_buffer_phase(&self) -> Result<()> {
        // Go to next song and back. This helps with missing metadata
        // for the first track in some configurations.
        next_song(&self.opts.service)?;
        thread::sleep(TIME_BETWEEN_SUBSEQUENT_DBUS_COMMANDS);
        previous_song(&self.opts.service)?;
        // Add a small time buffer before starting the playback properly.
        // This ensures that something starts playing, thus registering the
        // pulse audio sink. Also it avoids overflows when calculating the offset
        debug!("Begin pre-session phase");
        start_playback(&self.opts.service)?;
        thread::sleep(TIME_BEFORE_SESSION_START);
        stop_playback(&self.opts.service)?;
        debug!("Go to beginning of song");
        previous_song(&self.opts.service)?;
        thread::sleep(WAIT_TIME_BEFORE_FIRST_SONG);
        Ok(())
    }

    fn recording_phase(&mut self) -> Result<RecordingExitStatus> {
        info!("Starting playback.");
        start_playback(&self.opts.service)?;
        self.session.estimated_time_first_song = Some(self.recorder.time_since_start_secs());
        let mut time_last_dbus_signal = Instant::now();
        loop {
            let num_songs_before = self.session.songs.len();
            let playback_status = collect_dbus_info(&mut self.session, &self.opts.service)?;
            let num_songs_after = self.session.songs.len();
            if let RecordingStatus::Finished(exit_status) = playback_status {
                return Ok(exit_status);
            }
            // There should only be one new song if the delay between dbus signals is not too large, but you never know
            for song_index in num_songs_before..num_songs_after {
                self.add_new_song(self.session.songs[song_index].clone());
                time_last_dbus_signal = Instant::now();
            }
            if let Some(song) = self.session.songs.last() {
                let time_elapsed_after_estimated_song_ending = Instant::now()
                    .duration_since(time_last_dbus_signal)
                    .as_secs_f64()
                    - song.length;
                if time_elapsed_after_estimated_song_ending
                    > config::TIME_WITHOUT_DBUS_SIGNAL_BEFORE_STOPPING.as_secs_f64()
                {
                    return Ok(RecordingExitStatus::NoNewSongForTooLong);
                }
            }
        }
    }

    fn add_new_song(&self, song: Song) {
        self.song_sender.send(song).unwrap();
    }

    fn final_buffer_phase(&self) {
        debug!("Record final buffer for a few seconds");
        thread::sleep(TIME_AFTER_SESSION_END);
        info!("Recording finished.");
    }
}
