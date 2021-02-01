use regex::Regex;
use std::path::Path;
use std::process::Command;
use subprocess::{Exec, Popen};

use crate::config::{SINK_NAME, SINK_SOURCE_NAME};

pub fn start_recording(output_file: &Path) -> Popen {
    setup_recording();
    start_recording_command(output_file)
}

pub fn stop_recording(mut recording_handles: Popen) {
    recording_handles
        .terminate()
        .expect("Failed to terminate parec");
    println!("Stopped recording.");
}

fn start_recording_command(output_file: &Path) -> Popen {
    let parec_cmd = Exec::cmd("parec")
        .arg("-d")
        .arg(format!("{}.monitor", SINK_NAME))
        .arg("--file-format=wav")
        .arg(output_file.to_str().unwrap());
    parec_cmd.popen().expect("Failed to execute record command")
}

fn setup_recording() {
    if !check_sink_exists() {
        println!("Creating sink");
        create_sink();
    } else {
        println!("Sink already exists. Not creating sink");
    }
    let index = get_sink_input_index();
    if let Some(index_) = index {
        redirect_sink(index_);
    } else {
        // todo!(); // error
        println!("Failed to find sink index");
    }
}

fn redirect_sink(index: i32) {
    Command::new("pactl")
        .arg("move-sink-input")
        .arg(format!("{}", index))
        .arg(SINK_NAME)
        .output()
        .expect("Failed to execute sink redirection command");
}

fn check_sink_exists() -> bool {
    let output = Command::new("pacmd")
        .arg("list-sinks")
        .output()
        .expect("Failed to execute sink list command.");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.contains(SINK_NAME)
}

fn create_sink() {
    let output = Command::new("pactl")
        .arg("load-module")
        .arg("module-null-sink")
        .arg(format!("sink_name={}", SINK_NAME))
        .output()
        .expect("Failed to execute sink creation command.");
    assert!(output.status.success());
}

fn get_sink_input_index() -> Option<i32> {
    let output = Command::new("pacmd")
        .arg("list-sink-inputs")
        .output()
        .expect("Failed to execute list sink inputs command.");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stdout_without_newlines = stdout.replace("\n", "");
    // let re = Regex::new("index: ([0-9]*).*?media.name = \"(.*?)\"").unwrap();
    let re = Regex::new("index: ([0-9]*).*?media.name = \"(.*?)\"").unwrap();
    // let mat = re.find_iter(&stdout);
    let captures = re.captures_iter(&stdout_without_newlines);
    for capture in captures {
        let sink_source_index = capture.get(1).unwrap().as_str();
        let sink_source_name = capture.get(2).unwrap().as_str();
        if sink_source_name == SINK_SOURCE_NAME {
            return Some(
                sink_source_index
                    .parse::<i32>()
                    .expect("Integer conversion failed for sink index"),
            );
        }
    }
    None
}
