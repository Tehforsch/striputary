use std::fmt::Display;
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use anyhow::Context;
use anyhow::Result;
use log::info;
use serde::Deserialize;
use serde::Serialize;

use super::time::AudioTime;
use crate::config::OutputFormat;
use crate::consts::{self};
use crate::recording_session::RecordingSessionWithPath;
use crate::song::Song;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Cut {
    pub start_time_secs: f64,
    pub end_time_secs: f64,
    pub song: Song,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CutInfo {
    pub buffer_file: PathBuf,
    pub music_dir: PathBuf,
    pub cut: Cut,
    pub output_format: OutputFormat,
}

impl CutInfo {
    pub fn new(
        session: &RecordingSessionWithPath,
        song: &Song,
        start_time: AudioTime,
        end_time: AudioTime,
        output_format: OutputFormat,
    ) -> Self {
        let buffer_file = session.path.get_buffer_file();
        let music_dir = session.path.get_music_dir();
        CutInfo {
            buffer_file,
            music_dir,
            cut: Cut {
                start_time_secs: start_time.time,
                song: song.clone(),
                end_time_secs: end_time.time,
            },
            output_format: output_format,
        }
    }
}

fn add_metadata_arg_if_present<T: Display>(
    command: &mut Command,
    get_str: fn(&T) -> String,
    val: Option<&T>,
) {
    if let Some(val) = val {
        command.arg("-metadata").arg(get_str(val));
    }
}

pub fn get_cut_command(info: &CutInfo) -> Result<Command> {
    let difference = info.cut.end_time_secs - info.cut.start_time_secs;
    let target_file = info
        .cut
        .song
        .get_target_file(&info.music_dir, &info.output_format.format_ending());
    create_dir_all(target_file.parent().unwrap())
        .context("Failed to create subfolders of target file")?;
    info!(
        "Cutting song: {:.2}+{:.2}: {} to {}",
        info.cut.start_time_secs,
        difference,
        info.cut.song,
        target_file.to_str().unwrap()
    );
    let mut command = Command::new("ffmpeg");
    command
        .arg("-ss")
        .arg(format!("{}", info.cut.start_time_secs))
        .arg("-t")
        .arg(format!("{}", difference))
        .arg("-i")
        .arg(info.buffer_file.to_str().unwrap())
        .arg("-c:a")
        .arg(info.output_format.ffmpeg_libname())
        .arg("-b:a")
        .arg(format!("{}", consts::BITRATE));
    add_metadata_arg_if_present(
        &mut command,
        |title| format!("title='{}'", title),
        info.cut.song.title.as_ref(),
    );
    add_metadata_arg_if_present(
        &mut command,
        |album| format!("album='{}'", album),
        info.cut.song.album.as_ref(),
    );
    add_metadata_arg_if_present(
        &mut command,
        |artist| format!("artist='{}'", artist),
        info.cut.song.artist.as_ref(),
    );
    add_metadata_arg_if_present(
        &mut command,
        |artist| format!("albumartist='{}'", artist),
        info.cut.song.artist.as_ref(),
    );
    add_metadata_arg_if_present(
        &mut command,
        |track_number| format!("track={}", track_number),
        info.cut.song.track_number.as_ref(),
    );
    command
        .arg("-y")
        .arg(target_file.to_str().unwrap())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    Ok(command)
}
