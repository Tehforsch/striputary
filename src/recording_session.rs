use crate::song::Song;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::path::PathBuf;
use std::vec::Vec;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecordingSession {
    pub filename: PathBuf,
    pub songs: Vec<Song>,
    pub estimated_time_first_song: f64,
}

impl RecordingSession {
    pub fn new(path: &Path, estimated_time_first_song: f64) -> RecordingSession {
        RecordingSession {
            filename: path.to_owned(),
            estimated_time_first_song,
            songs: vec![],
        }
    }

    pub fn get_buffer_file(&self) -> PathBuf {
        Path::new(&self.filename.to_str().unwrap().replace("yaml", "wav")).into()
    }

    pub fn get_music_dir(&self) -> PathBuf {
        self.filename.parent().unwrap().join(Path::new("music"))
    }
}
