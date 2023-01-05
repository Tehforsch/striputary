use std::fs;
use std::path::Path;
use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;

use crate::config;
use crate::service_config::Service;

#[derive(Deserialize, Debug)]
pub struct ConfigFile {
    pub output_dir: PathBuf,
    pub service: Option<Service>,
}

impl ConfigFile {
    pub fn read() -> Result<Self> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("striputary").unwrap();
        let config_path = xdg_dirs.find_config_file(config::CONFIG_FILE_NAME);
        if let Some(config_path) = config_path {
            ConfigFile::from_file(&config_path)
        } else {
            Err(anyhow!("No config file found"))
        }
    }

    fn from_file(file: &Path) -> Result<ConfigFile> {
        let data =
            fs::read_to_string(file).context(format!("While reading config file at {:?}", file))?;
        let mut config_file: ConfigFile =
            serde_yaml::from_str(&data).context("Reading config file contents")?;
        config_file.output_dir = expanduser(&config_file.output_dir)?;
        Ok(config_file)
    }
}

pub fn expanduser(path: &Path) -> Result<PathBuf> {
    let expanded = shellexpand::tilde(path.to_str().unwrap());
    Path::new(&*expanded)
        .canonicalize()
        .context(format!("While reading {}", &expanded))
}
