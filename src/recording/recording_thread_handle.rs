use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use crate::config;
use crate::data_stream::DataStream;
use crate::recording_session::RecordingSession;
use crate::run_args::RunArgs;
use crate::song::Song;

use super::recording_thread::RecordingThread;

pub struct RecordingThreadHandle {
    handle: JoinHandle<Result<()>>,
    is_running: Arc<AtomicBool>,
    pub songs: DataStream<Song>,
    pub sessions: DataStream<RecordingSession>,
}

impl RecordingThreadHandle {
    pub fn new(run_args: &RunArgs) -> Self {
        let is_running = Arc::new(AtomicBool::new(true));
        let (song_sender, song_receiver) = channel();
        let (session_sender, session_receiver) = channel();
        let mut thread =
            RecordingThread::new(is_running.clone(), session_sender, song_sender, run_args);
        let handle = thread::spawn(move || thread.record_sessions_and_save_session_files());
        Self {
            handle,
            is_running,
            songs: DataStream::new(song_receiver),
            sessions: DataStream::new(session_receiver),
        }
    }

    pub fn update(&mut self) {
        self.songs.update(config::RECV_RECORDED_SONG_TIMEOUT);
        self.sessions.update(config::RECV_RECORDED_SESSION_TIMEOUT);
    }

    pub fn check_still_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    pub fn get_result(self) -> Result<()> {
        self.handle.join().unwrap()
    }
}
