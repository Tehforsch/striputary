use crate::path_utils::get_yaml_file;
use crate::recording_session::RecordingSession;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn save(session: &RecordingSession) -> Result<()> {
    let filename = get_yaml_file(&session.dir);
    let data = serde_yaml::to_string(session).context("Unable to convert session to yaml")?;
    fs::write(filename, data).context("Unable to write session file")
}

pub fn load(filename: &Path) -> Result<RecordingSession> {
    let data = fs::read_to_string(filename).context("Unable to read session file")?;
    serde_yaml::from_str(&data).context("Unable to load session file content.")
}
