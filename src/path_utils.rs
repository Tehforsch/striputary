use std::path::PathBuf;

use std::path::Path;

use crate::config::DEFAULT_BUFFER_FILE;

pub fn get_buffer_file(session_dir: &Path) -> PathBuf {
    session_dir.join(DEFAULT_BUFFER_FILE)
}

pub fn get_yaml_files(session_dir: &Path) -> Vec<PathBuf> {
    let mut files = vec![];
    let i = 0;
    loop {
        let file = session_dir.join(format!("{}.yaml", i));
        if file.is_file() {
            files.push(file);
        }
        else {
            break;
        }
    }
    files
}
