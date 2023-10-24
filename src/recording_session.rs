use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::vec::Vec;

use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

use crate::config;
use crate::recording::dbus_event::DbusEvent;
use crate::song::Song;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecordingSession {
    pub songs: Vec<Song>,
    pub estimated_time_first_song: Option<f64>,
}

impl RecordingSession {
    pub fn from_events(events: &[DbusEvent]) -> RecordingSession {
        RecordingSession {
            estimated_time_first_song: Some(0.0),
            songs: events
                .iter()
                .filter_map(|event| match event {
                    DbusEvent::NewSong(song) => Some(song.clone()),
                    DbusEvent::NewInvalidSong(_) => None,
                    DbusEvent::StatusChanged(_) => None,
                    DbusEvent::PlayerInformation(_) => None,
                })
                .collect(),
        }
    }

    pub fn save(&self, path: &SessionPath) -> Result<()> {
        let data = serde_yaml::to_string(self).context("Unable to convert session to yaml")?;
        fs::write(&path.get_yaml_file(), data).context("Unable to write session file")
    }

    pub fn from_file(filename: &Path) -> Result<Self> {
        let data = fs::read_to_string(filename).context("Unable to read session file")?;
        serde_yaml::from_str(&data).context("Unable to load session file content.")
    }
}

#[derive(Debug, Clone)]
pub struct SessionPath(pub PathBuf);

impl SessionPath {
    pub fn get_yaml_file(&self) -> PathBuf {
        self.0.join(config::DEFAULT_SESSION_FILE)
    }

    pub fn get_buffer_file(&self) -> PathBuf {
        self.0.join(config::DEFAULT_BUFFER_FILE)
    }

    pub fn get_music_dir(&self) -> PathBuf {
        self.0
            .parent()
            .unwrap()
            .join(Path::new(config::DEFAULT_MUSIC_DIR))
    }
}

#[derive(Debug, Clone)]
pub struct RecordingSessionWithPath {
    pub session: RecordingSession,
    pub path: SessionPath,
}

impl RecordingSessionWithPath {
    pub(crate) fn estimated_time_first_song(&self) -> f64 {
        0.0
    }

    pub fn load_from_dir(path: &Path) -> Result<Self> {
        Ok(Self {
            session: RecordingSession::from_file(&path.join(config::DEFAULT_SESSION_FILE))?,
            path: SessionPath(path.to_owned()),
        })
    }
}
