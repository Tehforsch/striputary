use std::path::Path;
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
use crate::recording::dbus::DbusEvent;
use crate::recording_session::RecordingSession;
use crate::recording_session::SessionPath;
use crate::song::Song;
use crate::Opts;

pub struct RecordingThread {
    is_running: Arc<AtomicBool>,
    song_sender: Sender<Song>,
    dbus_events: Vec<DbusEvent>,
    recorder: Recorder,
    dbus: DbusConnection,
    session_dir: SessionPath,
}

impl RecordingThread {
    pub fn new(
        is_running: Arc<AtomicBool>,
        song_sender: Sender<Song>,
        opts: &Opts,
        session_dir: &SessionPath,
    ) -> Self {
        let recorder = Recorder::start(opts, session_dir).unwrap();
        let recorder = Self {
            is_running,
            song_sender,
            dbus_events: vec![],
            recorder,
            dbus: DbusConnection::new(&opts.service),
            session_dir: session_dir.clone(),
        };
        recorder
    }

    pub fn record_new_session(mut self) -> Result<(RecordingStatus, RecordingSession)> {
        let status = self.polling_loop()?;
        self.stop()?;
        let session = RecordingSession::from_events(self.dbus_events);
        session.save(&self.session_dir);
        Ok((status, session))
    }

    fn stop(&mut self) -> Result<()> {
        self.is_running.store(false, Ordering::SeqCst);
        self.recorder.stop()
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
        loop {
            // collect here to make borrow checker happy
            let new_events: Vec<_> = self.dbus.get_new_events().collect();
            for event in new_events.iter() {
                match event {
                    DbusEvent::PlaybackStopped => {
                        return Ok(RecordingStatus::FinishedOrInterrupted);
                    }
                    _ => {}
                }
            }
            self.dbus_events.extend(new_events);
            self.check_new_songs();
        }
    }

    fn final_buffer_phase(&self) {
        info!(
            "Recording stopped. Recording final buffer for {} seconds.",
            TIME_AFTER_SESSION_END.as_secs()
        );
        thread::sleep(TIME_AFTER_SESSION_END);
        info!("Recording finished.");
    }

    fn check_new_songs(&self) {
        todo!()
    }
}
