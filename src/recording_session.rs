use std::path::PathBuf;
use std::time::Duration;
use std::vec::Vec;

use crate::song::Song;

pub struct RecordingSession {
    pub dir: PathBuf,
    pub timestamps: Vec<Duration>,
    pub songs: Vec<Song>,
}
