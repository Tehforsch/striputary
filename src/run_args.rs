use std::path::{Path, PathBuf};

use crate::service_config::ServiceConfig;

#[derive(Clone)]
pub struct RunArgs {
    pub session_dir: PathBuf,
    pub service_config: ServiceConfig,
}

impl RunArgs {
    pub fn new(session_dir: &Path, service_config: ServiceConfig) -> Self {
        Self {
            session_dir: session_dir.into(),
            service_config,
        }
    }

    pub fn get_yaml_file(&self, num: i32) -> PathBuf {
        self.session_dir.join(format!("{}.yaml", num))
    }

    pub fn get_buffer_file(&self, num: i32) -> PathBuf {
        self.session_dir.join(format!("{}.wav", num))
    }

    pub fn get_yaml_files(&self) -> Vec<PathBuf> {
        let mut files = vec![];
        for i in 0.. {
            let file = self.session_dir.join(format!("{}.yaml", i));
            if file.is_file() {
                files.push(file);
            } else {
                break;
            }
        }
        files
    }
}
