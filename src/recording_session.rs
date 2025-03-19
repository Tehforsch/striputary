use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::vec::Vec;

use anyhow::Context;
use anyhow::Result;
use chrono::Local;
use serde::Deserialize;
use serde::Serialize;

use crate::consts;
use crate::recording::dbus_event::DbusEvent;
use crate::recording::dbus_event::TimedDbusEvent;
use crate::recording::dbus_event::Timestamp;
use crate::song::Song;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecordingSession {
    pub songs: Vec<Song>,
    pub timestamps: Vec<Timestamp>,
}

impl RecordingSession {
    pub fn from_events(events: &[TimedDbusEvent]) -> RecordingSession {
        RecordingSession {
            songs: events
                .iter()
                .filter_map(|event| match &event.event {
                    DbusEvent::NewSong(song) => Some(song.clone()),
                    DbusEvent::NewInvalidSong => None,
                    DbusEvent::StatusChanged(_) => None,
                })
                .collect(),
            timestamps: events
                .iter()
                .filter_map(|event| match &event.event {
                    DbusEvent::NewSong(_) => Some(event.timestamp),
                    DbusEvent::NewInvalidSong => None,
                    DbusEvent::StatusChanged(_) => Some(event.timestamp),
                })
                .collect(),
        }
    }

    pub fn save(&self, path: &SessionPath) -> Result<()> {
        let data = serde_yaml::to_string(self).context("Unable to convert session to yaml")?;
        fs::write(&path.get_yaml_file(), data).context("Unable to write session file")
    }

    pub fn from_file(filename: &Path) -> Result<Self> {
        let data = fs::read_to_string(filename)
            .context(format!("Unable to read session file at {:?}", filename))?;
        serde_yaml::from_str(&data).context(format!(
            "Unable to load session file content of file at {:?}.",
            filename
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionPath(pub PathBuf);

impl std::fmt::Display for SessionPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string_lossy())
    }
}

impl<P: AsRef<Path>> From<P> for SessionPath {
    fn from(value: P) -> Self {
        Self(value.as_ref().to_owned())
    }
}

impl SessionPath {
    pub fn get_yaml_file(&self) -> PathBuf {
        self.0.join(consts::DEFAULT_SESSION_FILE)
    }

    pub fn get_buffer_file(&self) -> PathBuf {
        self.0.join(consts::DEFAULT_BUFFER_FILE)
    }

    pub fn get_music_dir(&self) -> PathBuf {
        self.0.join(Path::new(consts::DEFAULT_MUSIC_DIR))
    }
}

#[derive(Debug, Clone)]
pub struct RecordingSessionWithPath {
    pub session: RecordingSession,
    pub path: SessionPath,
}

impl RecordingSessionWithPath {
    pub fn load_from_dir(path: &Path) -> Result<Self> {
        Ok(Self {
            session: RecordingSession::from_file(&path.join(consts::DEFAULT_SESSION_FILE))?,
            path: SessionPath(path.to_owned()),
        })
    }
}

pub fn get_new_name(output_dir: &Path) -> PathBuf {
    let date_string = Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
    output_dir.join(date_string)
}
