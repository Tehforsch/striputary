use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread::{self};

use anyhow::Result;
use log::debug;
use log::info;

use super::dbus::DbusConnection;
use super::recording_status::RecordingStatus;
use super::Recorder;
use crate::config::TIME_AFTER_SESSION_END;
use crate::config::TIME_BEFORE_SESSION_START;
use crate::config::TIME_BETWEEN_SUBSEQUENT_DBUS_COMMANDS;
use crate::config::WAIT_TIME_BEFORE_FIRST_SONG;
use crate::gui::session_manager::SessionPath;
use crate::recording::dbus::DbusEvent;
use crate::recording_session::RecordingSession;
use crate::song::Song;
use crate::Opts;

pub struct RecordingThread {
    is_running: Arc<AtomicBool>,
    song_sender: Sender<Song>,
    session: RecordingSession,
    recorder: Recorder,
    dbus: DbusConnection,
}

impl RecordingThread {
    pub fn new(
        is_running: Arc<AtomicBool>,
        song_sender: Sender<Song>,
        opts: &Opts,
        session_dir: &SessionPath,
    ) -> Self {
        let session = RecordingSession::new(&session_dir.get_yaml_file());
        let recorder = Recorder::start(opts, session_dir).unwrap();
        let recorder = Self {
            is_running,
            song_sender,
            session,
            recorder,
            dbus: DbusConnection::new(&opts.service),
        };
        recorder
    }

    pub fn record_new_session(mut self) -> Result<(RecordingStatus, RecordingSession)> {
        let status = self.polling_loop()?;
        self.stop()?;
        Ok((status, self.session))
    }

    fn stop(&mut self) -> Result<()> {
        self.is_running.store(false, Ordering::SeqCst);
        self.recorder.stop()?;
        self.session.save()
    }

    fn polling_loop(&mut self) -> Result<RecordingStatus> {
        self.initial_buffer_phase()?;
        let status = self.recording_phase()?;
        self.final_buffer_phase();
        Ok(status)
    }

    fn initial_buffer_phase(&self) -> Result<()> {
        // Go to next song and back. This helps with missing metadata
        // for the first track in some configurations.
        self.dbus.next_song()?;
        thread::sleep(TIME_BETWEEN_SUBSEQUENT_DBUS_COMMANDS);
        self.dbus.previous_song()?;
        // Add a small time buffer before starting the playback properly.
        // This ensures that something starts playing, thus registering the
        // pulse audio sink. Also it avoids overflows when calculating the offset
        debug!("Begin pre-session phase");
        self.dbus.start_playback()?;
        thread::sleep(TIME_BEFORE_SESSION_START);
        self.dbus.stop_playback()?;
        debug!("Go to beginning of song");
        self.dbus.previous_song()?;
        thread::sleep(WAIT_TIME_BEFORE_FIRST_SONG);
        Ok(())
    }

    fn recording_phase(&mut self) -> Result<RecordingStatus> {
        info!("Starting playback.");
        self.dbus.start_playback()?;
        self.session.estimated_time_first_song = Some(self.recorder.time_since_start_secs());
        loop {
            // collect here to make borrow checker happy
            let new_events: Vec<_> = self.dbus.get_new_events().collect();
            for event in new_events {
                match event {
                    DbusEvent::NewSong(song) => {
                        // We get multiple dbus messages on every song change for every property that changes.
                        // Find out whether the song actually changed (or whether we havent recorded anything so far)
                        let is_different_song = self
                            .session
                            .songs
                            .last()
                            .map(|last_song| last_song != &song)
                            .unwrap_or(true);
                        if is_different_song {
                            self.add_new_song(song)?;
                        }
                    }
                    DbusEvent::PlaybackStopped => {
                        return Ok(RecordingStatus::FinishedOrInterrupted);
                    }
                }
            }
        }
    }

    fn add_new_song(&mut self, song: Song) -> Result<()> {
        info!("Now recording song: {}", song);
        self.session.songs.push(song.clone());
        self.session.save()?;
        self.song_sender.send(song).unwrap();
        Ok(())
    }

    fn final_buffer_phase(&self) {
        info!(
            "Recording stopped. Recording final buffer for {} seconds.",
            TIME_AFTER_SESSION_END.as_secs()
        );
        thread::sleep(TIME_AFTER_SESSION_END);
        info!("Recording finished.");
    }
}
