use std::thread::{self};
use std::time::Instant;

use anyhow::Result;
use log::info;

use super::dbus::DbusConnection;
use super::dbus_event::DbusEvent;
use super::dbus_event::TimedDbusEvent;
use super::dbus_event::Timestamp;
use super::AudioRecorder;
use super::RecordingStatus;
use crate::config::TIME_AFTER_SESSION_END;
use crate::recording::dbus_event::PlaybackStatus;
use crate::recording_session::RecordingSession;
use crate::recording_session::SessionPath;
use crate::song::Song;
use crate::Opts;

pub struct Recorder {
    dbus_events: Vec<TimedDbusEvent>,
    recorder: AudioRecorder,
    dbus: DbusConnection,
    session_dir: SessionPath,
    num_songs: usize,
    recording_start_time: Instant,
}

impl Recorder {
    pub fn new(opts: &Opts, session_dir: &SessionPath) -> Result<Self> {
        let recorder = AudioRecorder::start(opts, session_dir)?;
        let recorder = Self {
            dbus_events: vec![],
            recorder,
            dbus: DbusConnection::new(&opts.service),
            session_dir: session_dir.clone(),
            num_songs: 0,
            recording_start_time: Instant::now(),
        };
        Ok(recorder)
    }

    pub fn record_new_session(mut self) -> Result<(RecordingStatus, RecordingSession)> {
        let status = self.polling_loop()?;
        let session = self.get_session();
        session.save(&self.session_dir).unwrap();
        self.recorder.stop().unwrap();
        Ok((status, session))
    }

    fn get_session(&self) -> RecordingSession {
        RecordingSession::from_events(&self.dbus_events)
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
        info!("Now recording song: {}", song);
    }
}
