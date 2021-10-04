use anyhow::{Result};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration};


use crate::config;
use crate::run_args::RunArgs;
use crate::song::Song;

use super::recording_thread::RecordingThread;


pub struct RecordingThreadHandle {
    handle: JoinHandle<Result<()>>,
    is_running: Arc<AtomicBool>,
    receiver: Receiver<Song>,
}

impl RecordingThreadHandle {
    pub fn new(run_args: &RunArgs) -> Self {
        let is_running = Arc::new(AtomicBool::new(true));
        let (sender, receiver) = channel();
        let mut thread = RecordingThread::new(is_running.clone(), sender, run_args);
        let handle = thread::spawn(move || thread.record_sessions_and_save_session_files());
        Self {
            handle,
            is_running,
            receiver,
        }
    }

    pub fn check_still_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    pub fn get_result(self) -> Result<()> {
        self.handle.join().unwrap()
    }

    pub fn get_new_songs(&self) -> Option<Song> {
        self.receiver.recv_timeout(Duration::from_millis(config::RECV_RECORDED_SONG_TIMEOUT)).ok()
    }
}
