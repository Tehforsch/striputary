use std::path::PathBuf;

use std::path::Path;

use crate::config::DEFAULT_BUFFER_FILE;
use crate::config::DEFAULT_SESSION_FILE;

pub fn get_buffer_file(session_dir: &Path) -> PathBuf {
    session_dir.join(DEFAULT_BUFFER_FILE)
}

pub fn get_yaml_file(session_dir: &Path) -> PathBuf {
    session_dir.join(DEFAULT_SESSION_FILE)
}
