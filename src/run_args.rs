use std::path::Path;
use std::path::PathBuf;

use crate::config;
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

    pub fn get_yaml_file(&self) -> PathBuf {
        self.session_dir.join(config::DEFAULT_SESSION_FILE)
    }

    pub fn get_buffer_file(&self) -> PathBuf {
        self.session_dir.join(config::DEFAULT_BUFFER_FILE)
    }
}
