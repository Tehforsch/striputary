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
use crate::service_config::ServiceConfig;

pub fn start_recording(output_file: &Path, service_config: &ServiceConfig) -> Result<Popen> {
    setup_recording(service_config)?;
    start_recording_command(output_file)
}

pub fn stop_recording(mut recording_handles: Popen) -> Result<()> {
    recording_handles
        .terminate()
        .context("Failed to terminate parec while recording")?;
    println!("Stopped recording.");
    Ok(())
}

fn start_recording_command(output_file: &Path) -> Result<Popen> {
    let parec_cmd = Exec::cmd("parec")
        .arg("-d")
        .arg(format!("{}.monitor", STRIPUTARY_SINK_NAME))
        .arg("--file-format=wav")
        .arg(output_file.to_str().unwrap());
    parec_cmd
        .popen()
        .context("Failed to execute record command - is parec installed?")
}

fn setup_recording(service_config: &ServiceConfig) -> Result<()> {
    if !check_sink_exists()? {
        println!("Creating sink");
        create_sink()?;
    } else {
        println!("Sink already exists. Not creating sink");
    }
    let mb_index = get_sink_input_index(service_config)?;
    match mb_index {
        Some(index) => redirect_sink(index).map(|_| ()),
        None => Err(anyhow!("Failed to find sink index")),
    }
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

fn get_sink_input_index(service_config: &ServiceConfig) -> Result<Option<i32>> {
    let output = Command::new("pacmd")
        .arg("list-sink-inputs")
        .output()
        .context("Failed to execute list sink inputs command.")?;
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stdout_without_newlines = stdout.replace("\n", "");
    let re = Regex::new("index: ([0-9]*).*?media.name = \"(.*?)\"").unwrap();
    let captures = re.captures_iter(&stdout_without_newlines);
    let mut temp = captures.filter(|capture| {
        get_sink_source_name_from_pacmd_output_capture(capture)
            .map(|name| name == service_config.sink_name)
            .unwrap_or(false)
    });
    temp.next()
        .map(|capture| get_sink_index_from_pacmd_output_capture(&capture))
        .transpose()
}

fn get_sink_index_from_pacmd_output_capture(capture: &Captures) -> Result<i32> {
    let sink_source_index = capture.get(1).context("Invalid line")?.as_str();
    sink_source_index
        .parse::<i32>()
        .context("Integer conversion failed for sink index")
}

fn get_sink_source_name_from_pacmd_output_capture<'a>(capture: &'a Captures) -> Result<&'a str> {
    Ok(capture.get(2).context("Invalid line")?.as_str())
}
