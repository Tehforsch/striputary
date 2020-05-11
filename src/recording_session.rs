use crate::config::DEFAULT_BUFFER_FILE;
use crate::song::Song;
use std::path::PathBuf;
use std::time::Duration;
use std::vec::Vec;

pub struct RecordingSession {
    pub dir: PathBuf,
    pub timestamps: Vec<Duration>,
    pub songs: Vec<Song>,
}

impl RecordingSession {
    pub fn get_buffer_file(&self) -> PathBuf {
        self.dir.join(DEFAULT_BUFFER_FILE)
    }
}
