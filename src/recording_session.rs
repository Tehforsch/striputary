use crate::config::DEFAULT_BUFFER_FILE;
use crate::song::Song;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use std::vec::Vec;

#[derive(Serialize, Deserialize, Debug)]
pub struct RecordingSession {
    pub dir: PathBuf,
    pub timestamps: Vec<u128>,
    pub songs: Vec<Song>,
}

impl RecordingSession {
    pub fn get_buffer_file(&self) -> PathBuf {
        self.dir.join(DEFAULT_BUFFER_FILE)
    }
}
