use std::path::PathBuf;

use std::path::Path;

pub fn get_yaml_file(session_dir: &Path, num: i32) -> PathBuf {
    session_dir.join(format!("{}.yaml", num)).into()
}

pub fn get_buffer_file(session_dir: &Path, num: i32) -> PathBuf {
    session_dir.join(format!("{}.wav", num)).into()
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
