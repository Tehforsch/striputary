use crate::config::DEFAULT_BUFFER_FILE;
use crate::song::Song;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::path::PathBuf;
use std::vec::Vec;

#[derive(Serialize, Deserialize, Debug)]
pub struct RecordingSession {
    pub dir: PathBuf,
    pub songs: Vec<Song>,
    pub estimated_time_first_song: f64,
}

impl RecordingSession {
    pub fn new(path: &Path, estimated_time_first_song: f64) -> RecordingSession {
        RecordingSession {
            dir: path.to_owned(),
            estimated_time_first_song,
            songs: vec![],
        }
    }

    pub fn get_buffer_file(&self) -> PathBuf {
        self.dir.join(DEFAULT_BUFFER_FILE)
    }

    pub fn get_music_dir(&self) -> PathBuf {
        self.dir.join(Path::new("music"))
    }
}
