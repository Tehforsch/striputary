use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::thread::{self};

use anyhow::Result;

use super::recording_status::RecordingExitStatus;
use super::recording_thread::RecordingThread;
use crate::config;
use crate::data_stream::DataStream;
use crate::recording_session::RecordingSession;
use crate::song::Song;
use crate::Opts;

pub struct AsyncRecorder {
    handle: JoinHandle<Result<(RecordingExitStatus, RecordingSession)>>,
    is_running: Arc<AtomicBool>,
    pub songs: DataStream<Song>,
}

impl AsyncRecorder {
    pub fn new(opts: &Opts) -> Self {
        let is_running = Arc::new(AtomicBool::new(true));
        let (song_sender, song_receiver) = channel();
        let thread = RecordingThread::new(is_running.clone(), song_sender, opts);
        let handle = thread::spawn(move || thread.record_new_session());
        Self {
            handle,
            is_running,
            songs: DataStream::new(song_receiver),
        }
    }

    pub fn update(&mut self) {
        self.songs.update(config::RECV_RECORDED_SONG_TIMEOUT);
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    pub fn get_result(self) -> Result<(RecordingExitStatus, RecordingSession)> {
        self.handle.join().unwrap()
    }
}
