use crate::recording_session::RecordingSession;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub fn save(filename: &Path, session: RecordingSession) {
    let data = serde_yaml::to_string(&session).expect("Unable to convert session to yaml");
    fs::write(filename, data).expect("Unable to write session file");
}

pub fn load(filename: &Path) -> RecordingSession {
    let data = fs::read_to_string(filename).expect("Unable to read session file");
    let session: RecordingSession =
        serde_yaml::from_str(&data).expect("Unable to load session file content.");
    session
}
