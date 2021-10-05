use anyhow::{anyhow, Result};

use crate::song::Song;
use crate::yaml_session;
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

pub fn load_sessions(session_dir: &Path) -> Result<Vec<RecordingSession>> {
    let files = get_yaml_files(session_dir);
    if files.is_empty() {
        return Err(anyhow!("No session files found!"));
    }
    files
        .iter()
        .map(|yaml_file| yaml_session::load(&yaml_file))
        .collect::<Result<Vec<_>>>()
}

pub fn get_yaml_files(session_dir: &Path) -> Vec<PathBuf> {
    let mut files = vec![];
    for i in 0.. {
        let file = session_dir.join(format!("{}.yaml", i));
        if file.is_file() {
            files.push(file);
        } else {
            break;
        }
    }
    files
}
