use std::fs::create_dir_all;
use std::path::Path;
use std::process::Command;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use regex::Captures;
use regex::Regex;
use subprocess::Exec;
use subprocess::Popen;

use crate::config::STRIPUTARY_SINK_NAME;
use crate::run_args::RunArgs;
use crate::service_config::ServiceConfig;

pub struct Recorder {
    process: Popen,
}

impl Recorder {
    pub fn start(opts: &RunArgs) -> Result<Self> {
        setup_recording(&opts)?;
        Ok(Self {
            process: start_recording_process(&opts.get_buffer_file())?,
        })
    }

    pub fn stop(&mut self) -> Result<()> {
        self.process
            .terminate()
            .context("Failed to terminate parec while recording")?;
        println!("Stopped recording.");
        Ok(())
    }
}

fn start_recording_process(buffer_file: &Path) -> Result<Popen> {
    let parec_cmd = Exec::cmd("parec")
        .arg("-d")
        .arg(format!("{}.monitor", STRIPUTARY_SINK_NAME))
        .arg("--file-format=wav")
        .arg(buffer_file.to_str().unwrap());
    parec_cmd
        .popen()
        .context("Failed to execute record command - is parec installed?")
}

fn setup_recording(opts: &RunArgs) -> Result<()> {
    create_dir_all(&opts.session_dir).context("Failed to create session directory")?;
    if opts.get_buffer_file().exists() {
        return Err(anyhow!(
            "Buffer file already exists, not recording a new session."
        ));
    }
    if !check_sink_exists()? {
        println!("Creating sink");
        create_sink()?;
    } else {
        println!("Sink already exists. Not creating sink");
    }
    let index = get_sink_input_index(&opts.service_config)?;
    redirect_sink(index)
}

fn redirect_sink(index: i32) -> Result<()> {
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

fn check_sink_exists() -> Result<bool> {
    let output = Command::new("pacmd")
        .arg("list-sinks")
        .output()
        .context("Failed to execute sink list command (pacmd list-sinks) - is pacmd installed?.")?;
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.contains(STRIPUTARY_SINK_NAME))
}

fn create_sink() -> Result<()> {
    let output = Command::new("pactl")
        .arg("load-module")
        .arg("module-null-sink")
        .arg(format!("sink_name={}", STRIPUTARY_SINK_NAME))
        .output()
        .context("Failed to execute sink creation command.")?;
    assert!(output.status.success());
    Ok(())
}

fn get_sink_input_index(service_config: &ServiceConfig) -> Result<i32> {
    let output = Command::new("pacmd")
        .arg("list-sink-inputs")
        .output()
        .context("Failed to execute list sink inputs command.")?;
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout).replace('\n', "");
    let re = Regex::new("index: ([0-9]*).*?media.name = \"(.*?)\"").unwrap();
    for capture in re.captures_iter(&stdout) {
        let name = get_sink_source_name_from_capture(&capture);
        if let Ok(name) = name {
            if name == service_config.sink_name {
                return get_sink_index_from_capture(&capture);
            }
        }
    }
    Err(anyhow!("Failed to get sink input index"))
}

fn get_sink_index_from_capture(capture: &Captures) -> Result<i32> {
    let sink_source_index = capture.get(1).context("Invalid line")?.as_str();
    sink_source_index
        .parse::<i32>()
        .context("Integer conversion failed for sink index")
}

fn get_sink_source_name_from_capture<'a>(capture: &'a Captures) -> Result<&'a str> {
    Ok(capture.get(2).context("Invalid line")?.as_str())
}
