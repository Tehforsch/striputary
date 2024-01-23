use std::fmt::Display;
use std::fs::create_dir_all;
use std::path::Path;
use std::process::Command;
use std::str::FromStr;
use std::time::Instant;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use log::debug;
use regex::Captures;
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use subprocess::Exec;
use subprocess::Popen;

use crate::config::STRIPUTARY_SINK_NAME;
use crate::recording_session::SessionPath;
use crate::service::Service;
use crate::Opts;

#[derive(Copy, Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SoundServer {
    #[default]
    Pulseaudio,
    Pipewire,
}

impl FromStr for SoundServer {
    type Err = serde_yaml::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // This is quite ugly, but ensures that the config file string representation
        // is the same as in the command line options (which uses the FromStr impl),
        // without adding any additional dependencies
        serde_yaml::from_str(s)
    }
}

impl Display for SoundServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Similar to the from_str implementation, this is ugly but consistent.
        write!(f, "{}", serde_yaml::to_string(self).unwrap())
    }
}

pub struct AudioRecorder {
    process: Popen,
    start_time: Instant,
}

impl AudioRecorder {
    pub fn start(opts: &Opts, session_path: &SessionPath) -> Result<Self> {
        opts.sound_server.setup_recording(&opts, session_path)?;
        Ok(Self {
            process: opts
                .sound_server
                .start_recording_process(&session_path.get_buffer_file())?,
            start_time: Instant::now(),
        })
    }

    pub fn stop(&mut self) -> Result<()> {
        self.process
            .terminate()
            .context("Failed to terminate parec while recording")?;
        Ok(())
    }

    pub fn time_since_start_secs(&self) -> f64 {
        Instant::now().duration_since(self.start_time).as_millis() as f64 / 1000.0
    }
}

impl SoundServer {
    fn start_recording_process(&self, buffer_file: &Path) -> Result<Popen> {
        let parec_cmd = Exec::cmd("parec")
            .arg("-d")
            .arg(format!("{}.monitor", STRIPUTARY_SINK_NAME))
            .arg("--file-format=wav")
            .arg(buffer_file.to_str().unwrap());
        parec_cmd
            .popen()
            .context("Failed to execute record command - is parec installed?")
    }

    fn setup_recording(&self, opts: &Opts, session_path: &SessionPath) -> Result<()> {
        create_dir_all(&session_path.0).context("Failed to create session directory")?;
        if session_path.get_buffer_file().exists() {
            return Err(anyhow!(
                "Buffer file already exists, not recording a new session."
            ));
        }
        if !self.check_sink_exists()? {
            debug!("Creating sink");
            self.create_sink()?;
        } else {
            debug!("Sink already exists. Not creating sink");
        }
        let index = self.get_sink_input_index(&opts.service)?;
        self.redirect_sink(index)
    }

    fn redirect_sink(&self, index: i32) -> Result<()> {
        Command::new("pactl")
            .arg("move-sink-input")
            .arg(format!("{}", index))
            .arg(STRIPUTARY_SINK_NAME)
            .output()
            .context(
                "Failed to execute sink redirection via pactl move-sink-input - is pactl installed?",
            )?;
        Ok(())
    }

    fn check_sink_exists(&self) -> Result<bool> {
        let output = Command::new("pactl")
            .arg("list")
            .arg("sinks")
            .output()
            .context(
                "Failed to execute sink list command (pacmd list-sinks) - is pacmd installed?.",
            )?;
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.contains(STRIPUTARY_SINK_NAME))
    }

    fn create_sink(&self) -> Result<()> {
        let output = Command::new("pactl")
            .arg("load-module")
            .arg("module-null-sink")
            .arg(format!("sink_name={}", STRIPUTARY_SINK_NAME))
            .output()
            .context("Failed to execute sink creation command.")?;
        assert!(output.status.success());
        Ok(())
    }

    fn get_sink_input_regex(&self) -> Regex {
        match self {
            SoundServer::Pipewire => {
                Regex::new("Sink Input #([0-9]*).*?media.name = \"(.*?)\"").unwrap()
            }
            SoundServer::Pulseaudio => {
                Regex::new("index: ([0-9]*).*?media.name = \"(.*?)\"").unwrap()
            }
        }
    }

    fn get_sink_input_index(&self, service: &Service) -> Result<i32> {
        let output = Command::new("pactl")
            .arg("list")
            .arg("sink-inputs")
            .output()
            .context("Failed to execute list sink inputs command.")?;
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout).replace('\n', "");
        let re = self.get_sink_input_regex();
        for capture in re.captures_iter(&stdout) {
            let name = self.get_sink_source_name_from_capture(&capture);
            if let Ok(name) = name {
                if name == service.sink_name() {
                    return self.get_sink_index_from_capture(&capture);
                }
            }
        }
        Err(anyhow!("Failed to get sink input index"))
    }

    fn get_sink_index_from_capture(&self, capture: &Captures) -> Result<i32> {
        let sink_source_index = capture.get(1).context("Invalid line")?.as_str();
        sink_source_index
            .parse::<i32>()
            .context("Integer conversion failed for sink index")
    }

    fn get_sink_source_name_from_capture<'a>(&self, capture: &'a Captures) -> Result<&'a str> {
        Ok(capture.get(2).context("Invalid line")?.as_str())
    }
}
