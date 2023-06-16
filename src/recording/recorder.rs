use std::path::Path;
use std::process::Command;
use std::process::Output;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use regex::Captures;
use regex::Regex;
use subprocess::Exec;
use subprocess::Popen;

use crate::config::STRIPUTARY_MONITOR_SINK_DESCRIPTION;
use crate::config::STRIPUTARY_MONITOR_SINK_NAME;
use crate::config::STRIPUTARY_SINK_DESCRIPTION;
use crate::config::STRIPUTARY_SINK_NAME;
use crate::service_config::ServiceConfig;
use crate::sink_type::SinkType;

fn run_command_and_assert_success(command: &mut Command) -> Result<Output> {
    let output = command.output()?;
    assert!(output.status.success());
    Ok(output)
}

pub fn start_recording(
    output_file: &Path,
    service_config: &ServiceConfig,
    sink_type: SinkType,
) -> Result<Popen> {
    setup_recording(service_config, sink_type)?;
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

fn setup_recording(service_config: &ServiceConfig, sink_type: SinkType) -> Result<()> {
    if check_sink_exists()? {
        remove_sink()?;
    }
    let output_sink_name = create_sink(sink_type)?;
    let mb_index = get_sink_input_index(service_config)?;
    match mb_index {
        Some(index) => redirect_sink(index, output_sink_name).map(|_| ()),
        None => Err(anyhow!(
            "Failed to find sink index for service: {}",
            service_config.sink_name
        )),
    }
}

fn redirect_sink(index: i32, output_sink_name: &str) -> Result<()> {
    run_command_and_assert_success(
        Command::new("pactl")
            .arg("move-sink-input")
            .arg(format!("{}", index))
            .arg(output_sink_name),
    )
    .map(|_| ())
    .context("Failed to execute sink redirection via pactl move-sink-input - is pactl installed?")
}

fn check_sink_exists() -> Result<bool> {
    let output = run_command_and_assert_success(Command::new("pacmd").arg("list-sinks"))
        .context("Failed to execute sink list command (pacmd list-sinks) - is pacmd installed?.")?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.contains(STRIPUTARY_SINK_NAME))
}

fn get_default_sink_for_monitor() -> String {
    String::from_utf8(
        run_command_and_assert_success(Command::new("pactl").arg("get-default-sink"))
            .expect("Failed to get name of default sink for monitor.")
            .stdout,
    )
    .expect("Failed to decode output of pactl get-default-sink")
}

fn remove_sink() -> Result<()> {
    run_command_and_assert_success(
        Command::new("pacmd")
            .arg("unload-module")
            .arg("module-null-sink"),
    )
    .map(|_| ())
}

fn create_sink(sink_type: SinkType) -> Result<&'static str> {
    let output = Command::new("pacmd")
        .arg("load-module")
        .arg("module-null-sink")
        .arg(format!("sink_name={}", STRIPUTARY_SINK_NAME))
        .arg(format!(
            "sink_properties=device.description={}",
            STRIPUTARY_SINK_DESCRIPTION
        ))
        .output()
        .context("Failed to execute sink creation command.")?;
    assert!(output.status.success());
    if let SinkType::Monitor = sink_type {
        create_monitor_sink()?;
        Ok(STRIPUTARY_MONITOR_SINK_NAME)
    } else {
        Ok(STRIPUTARY_SINK_NAME)
    }
}

fn create_monitor_sink() -> Result<()> {
    let output = Command::new("pacmd")
        .arg("load-module")
        .arg("module-combine-sink")
        .arg(format!("sink_name={}", STRIPUTARY_MONITOR_SINK_NAME))
        .arg(format!(
            "sink_properties=device.description={}",
            STRIPUTARY_MONITOR_SINK_DESCRIPTION
        ))
        .arg(format!(
            "slaves={},{}",
            STRIPUTARY_SINK_NAME,
            get_default_sink_for_monitor()
        ))
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
