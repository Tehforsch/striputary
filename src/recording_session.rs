use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::vec::Vec;

use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

use crate::config;
use crate::song::Song;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecordingSession {
    #[serde(skip_serializing, skip_deserializing)]
    pub filename: PathBuf,
    pub songs: Vec<Song>,
    pub estimated_time_first_song: Option<f64>,
}

impl RecordingSession {
    pub fn new(path: &Path) -> RecordingSession {
        RecordingSession {
            filename: path.to_owned(),
            estimated_time_first_song: Some(0.0),
            songs: vec![],
        }
    }

    pub fn get_buffer_file(&self) -> PathBuf {
        self.filename
            .parent()
            .unwrap()
            .join(config::DEFAULT_BUFFER_FILE)
    }

    pub fn get_music_dir(&self) -> PathBuf {
        self.filename
            .parent()
            .unwrap()
            .join(Path::new(config::DEFAULT_MUSIC_DIR))
    }

    pub fn save(&self) -> Result<()> {
        let data = serde_yaml::to_string(self).context("Unable to convert session to yaml")?;
        fs::write(&self.filename, data).context("Unable to write session file")
    }

    pub fn from_file(filename: &Path) -> Result<Self> {
        let data = fs::read_to_string(filename).context("Unable to read session file")?;
        let mut session: RecordingSession =
            serde_yaml::from_str(&data).context("Unable to load session file content.")?;
        session.filename = filename.into();
        Ok(session)
    }

    pub fn from_parent_dir(dirname: &Path) -> Result<Self> {
        Self::from_file(&dirname.join(config::DEFAULT_SESSION_FILE))
    }
}
