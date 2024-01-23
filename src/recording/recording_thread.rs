use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread::{self};
use std::time::Instant;

use anyhow::Result;
use log::info;

use super::dbus::DbusConnection;
use super::dbus_event::DbusEvent;
use super::dbus_event::TimedDbusEvent;
use super::dbus_event::Timestamp;
use super::recording_status::RecordingStatus;
use super::Recorder;
use crate::config::TIME_AFTER_SESSION_END;
use crate::recording::dbus_event::PlaybackStatus;
use crate::recording_session::RecordingSession;
use crate::recording_session::SessionPath;
use crate::song::Song;
use crate::Opts;

pub struct RecordingThread {
    is_running: Arc<AtomicBool>,
    song_sender: Sender<Song>,
    dbus_events: Vec<TimedDbusEvent>,
    recorder: Recorder,
    dbus: DbusConnection,
    session_dir: SessionPath,
    num_songs: usize,
    recording_start_time: Instant,
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
            num_songs: 0,
            recording_start_time: Instant::now(),
        };
        recorder
    }

    pub fn record_new_session(mut self) -> Result<(RecordingStatus, RecordingSession)> {
        let status = self.polling_loop()?;
        self.stop()?;
        let session = self.get_session();
        session.save(&self.session_dir).unwrap();
        Ok((status, session))
    }

    fn get_session(&self) -> RecordingSession {
        RecordingSession::from_events(&self.dbus_events)
    }

    fn stop(&mut self) -> Result<()> {
        self.is_running.store(false, Ordering::SeqCst);
        self.recorder.stop()
    }

    fn polling_loop(&mut self) -> Result<RecordingStatus> {
        let status = self.recording_loop()?;
        self.record_final_buffer();
        Ok(status)
    }

    fn recording_loop(&mut self) -> Result<RecordingStatus> {
        loop {
            // collect here to make borrow checker happy
            let new_events: Vec<_> = self
                .dbus
                .get_new_events()
                .map(|(event, instant)| {
                    let duration = instant.duration_since(self.recording_start_time);
                    TimedDbusEvent {
                        event,
                        timestamp: Timestamp::from_duration(duration),
                    }
                })
                .collect();
            if !new_events.is_empty() {
                self.dbus_events.extend(new_events.clone());
                for event in new_events {
                    match event.event {
                        DbusEvent::StatusChanged(PlaybackStatus::Paused) => {
                            return Ok(RecordingStatus::FinishedOrInterrupted);
                        }
                        _ => {}
                    }
                }
                self.log_new_songs();
            }
        }
    }

    fn record_final_buffer(&self) {
        info!(
            "Recording stopped. Recording final buffer for {} seconds.",
            TIME_AFTER_SESSION_END.as_secs()
        );
        thread::sleep(TIME_AFTER_SESSION_END);
        info!("Recording finished.");
    }

    fn log_new_songs(&mut self) {
        // Because we want the [DbusEvent] -> RecordingSession mapping
        // to be pure, we compute it everytime we get a new dbus event
        // and then check if any new songs have been recorded in the
        // dbus events. This is slightly awkward but preferable to having
        // lots of mangled state.
        let session = self.get_session();
        if session.songs.len() > self.num_songs {
            for song in session.songs[self.num_songs..].iter() {
                self.log_new_song(song);
            }
            session.save(&self.session_dir).unwrap();
        }
        self.num_songs = session.songs.len();
    }

    fn log_new_song(&mut self, song: &Song) {
        self.song_sender.send(song.clone()).unwrap();
        info!("Now recording song: {}", song);
    }
}
