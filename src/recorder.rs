use anyhow::{anyhow, Context, Result};
use regex::{Captures, Regex};
use std::path::Path;
use std::process::Command;
use subprocess::{Exec, Popen};

use crate::config::{SINK_NAME, SINK_SOURCE_NAME};

pub fn start_recording(output_file: &Path) -> Result<Popen> {
    setup_recording()?;
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
        .arg(format!("{}.monitor", SINK_NAME))
        .arg("--file-format=wav")
        .arg(output_file.to_str().unwrap());
    parec_cmd
        .popen()
        .context("Failed to execute record command")
}

fn setup_recording() -> Result<()> {
    if !check_sink_exists()? {
        println!("Creating sink");
        create_sink()?;
    } else {
        println!("Sink already exists. Not creating sink");
    }
    let mb_index = get_sink_input_index()?;
    match mb_index {
        Some(index) => redirect_sink(index).map(|_| ()),
        None => Err(anyhow!("Failed to find sink index")),
    }
}

fn redirect_sink(index: i32) -> Result<()> {
    Command::new("pactl")
        .arg("move-sink-input")
        .arg(format!("{}", index))
        .arg(SINK_NAME)
        .output()
        .context("Failed to execute sink redirection via pactl move-sink-input")?;
    Ok(())
}

fn check_sink_exists() -> Result<bool> {
    let output = Command::new("pacmd")
        .arg("list-sinks")
        .output()
        .context("Failed to execute sink list command (pacmd list-sinks).")?;
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.contains(SINK_NAME))
}

fn create_sink() -> Result<()> {
    let output = Command::new("pactl")
        .arg("load-module")
        .arg("module-null-sink")
        .arg(format!("sink_name={}", SINK_NAME))
        .output()
        .context("Failed to execute sink creation command.")?;
    assert!(output.status.success());
    Ok(())
}

fn get_sink_input_index() -> Result<Option<i32>> {
    let output = Command::new("pacmd")
        .arg("list-sink-inputs")
        .output()
        .context("Failed to execute list sink inputs command.")?;
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stdout_without_newlines = stdout.replace("\n", "");
    let re = Regex::new("index: ([0-9]*).*?media.name = \"(.*?)\"").unwrap();
    let captures = re.captures_iter(&stdout_without_newlines);
    captures
        .filter(|capture| {
            get_sink_source_name_from_pacmd_output_capture(capture)
                .map(|name| name == SINK_SOURCE_NAME)
                .unwrap_or(false)
        })
        .next()
        .map(|capture| get_sink_index_from_pacmd_output_capture(&capture))
        .transpose()
    // for capture in captures {
    //     if sink_source_name == SINK_SOURCE_NAME {
    //         ;
    //     }
    // }
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
