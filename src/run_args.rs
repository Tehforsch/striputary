use std::path::Path;
use std::path::PathBuf;

use crate::config;
use crate::service_config::ServiceConfig;
use crate::sink_type::SinkType;

#[derive(Clone)]
pub struct RunArgs {
    pub session_dir: PathBuf,
    pub service_config: ServiceConfig,
    pub sink_type: SinkType,
}

impl RunArgs {
    pub fn new(session_dir: &Path, service_config: ServiceConfig, sink_type: SinkType) -> Self {
        Self {
            session_dir: session_dir.into(),
            service_config,
            sink_type,
        }
    }

    pub fn get_yaml_file(&self) -> PathBuf {
        self.session_dir.join(config::DEFAULT_SESSION_FILE)
    }

    pub fn get_buffer_file(&self) -> PathBuf {
        self.session_dir.join(config::DEFAULT_BUFFER_FILE)
    }
}
