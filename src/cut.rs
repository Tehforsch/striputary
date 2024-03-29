use std::fmt::Display;
use std::fs::create_dir_all;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use anyhow::Context;
use anyhow::Result;

use crate::audio_excerpt::AudioExcerpt;
use crate::audio_time::AudioTime;
use crate::config::MAX_OFFSET;
use crate::config::MIN_OFFSET;
use crate::config::NUM_OFFSETS_TO_TRY;
use crate::config::READ_BUFFER;
use crate::config::{self};
use crate::excerpt_collection::ExcerptCollection;
use crate::excerpt_collection::NamedExcerpt;
use crate::recording_session::RecordingSession;
use crate::song::Song;
use crate::wav::extract_audio;

pub struct CutInfo {
    pub song: Song,
    buffer_file: PathBuf,
    music_dir: PathBuf,
    start_time: AudioTime,
    end_time: AudioTime,
    num_in_recording: usize,
}

impl CutInfo {
    pub fn new(
        session: &RecordingSession,
        song: Song,
        start_time: AudioTime,
        end_time: AudioTime,
        num_in_recording: usize,
    ) -> Self {
        let buffer_file = session.get_buffer_file();
        let music_dir = session.get_music_dir();
        CutInfo {
            song,
            buffer_file,
            music_dir,
            start_time,
            end_time,
            num_in_recording,
        }
    }
}

fn get_excerpt(buffer_file_name: &Path, cut_time: f64) -> Option<AudioExcerpt> {
    let listen_start_time = cut_time + MIN_OFFSET - READ_BUFFER;
    let listen_end_time = cut_time + MAX_OFFSET + READ_BUFFER;
    extract_audio(buffer_file_name, listen_start_time, listen_end_time).ok()
}

fn get_cut_timestamps_from_song_lengths(
    songs: &[Song],
    estimated_time_first_song: f64,
) -> Vec<f64> {
    songs
        .iter()
        .scan(estimated_time_first_song, |acc, song| {
            let result = Some(*acc);
            *acc += song.length;
            result
        })
        .collect()
}

fn determine_cut_offset(audio_excerpts: &[AudioExcerpt], cut_timestamps: &[f64]) -> f64 {
    // We can assume that some of the songs begin or end with silence.
    // If that is the case then the offset of the cuts should be chosen by finding an offset that
    // puts as many of the cuts at positions where the recording is silent. In other words, the offset is given by
    // the local minimum of the convolution of the volume with a sum of dirac deltas at every cut position.
    let mut min: Option<(f64, f64)> = None;
    for i in 0..NUM_OFFSETS_TO_TRY {
        let offset =
            (i as f64) / (NUM_OFFSETS_TO_TRY as f64) * (MAX_OFFSET - MIN_OFFSET) + MIN_OFFSET;
        let total_volume: f64 = cut_timestamps
            .iter()
            .zip(audio_excerpts.iter())
            .map(|(cut_time, audio_excerpt)| audio_excerpt.get_volume_at(cut_time + offset))
            .sum();
        if let Some((min_volume, _)) = min {
            if total_volume < min_volume {
                min = Some((total_volume, offset));
            }
        } else {
            min = Some((total_volume, offset));
        };
    }
    let cut_quality_estimate = min.unwrap().0 / (audio_excerpts.len() as f64);
    println!("Av. volume at cuts: {:.3}", cut_quality_estimate);
    min.unwrap().1
}

pub fn get_excerpt_collection(session: RecordingSession) -> ExcerptCollection {
    let (excerpts, songs) = get_all_valid_excerpts_and_songs(&session);
    let timestamps =
        get_cut_timestamps_from_song_lengths(&songs, session.estimated_time_first_song);
    let offset_guess = determine_cut_offset(&excerpts, &timestamps);
    let excerpts: Vec<NamedExcerpt> = excerpts
        .into_iter()
        .enumerate()
        .map(|(num, excerpt)| {
            let song_before = if num == 0 { None } else { songs.get(num) };
            NamedExcerpt {
                excerpt,
                song_before: song_before.cloned(),
                song_after: songs.get(num).cloned(),
                num,
            }
        })
        .collect();
    ExcerptCollection {
        session,
        excerpts,
        offset_guess,
    }
}

fn get_all_valid_excerpts_and_songs(session: &RecordingSession) -> (Vec<AudioExcerpt>, Vec<Song>) {
    let mut audio_excerpts = Vec::new();
    let mut valid_songs = Vec::new();
    let mut cut_time = session.estimated_time_first_song;
    for song in session.songs.iter() {
        let audio_excerpt = get_excerpt(&session.get_buffer_file(), cut_time);
        if let Some(excerpt) = audio_excerpt {
            audio_excerpts.push(excerpt);
            valid_songs.push(song.clone());
        } else {
            break;
        }
        cut_time += song.length;
    }
    let audio_excerpt_after_last_song = get_excerpt(&session.get_buffer_file(), cut_time);
    if let Some(audio_excerpt_after_last_song) = audio_excerpt_after_last_song {
        audio_excerpts.push(audio_excerpt_after_last_song);
    }
    (audio_excerpts, valid_songs)
}

fn add_metadata_arg_if_present<T: Display>(
    command: &mut Command,
    get_str: fn(&T) -> String,
    val: Option<&T>,
) {
    if let Some(val) = val {
        command.arg("-metadata").arg(get_str(&val));
    }
}

pub fn cut_song(info: &CutInfo) -> Result<()> {
    let difference = info.end_time.time - info.start_time.time;
    let target_file = info
        .song
        .get_target_file(&info.music_dir, info.num_in_recording);
    create_dir_all(target_file.parent().unwrap())
        .context("Failed to create subfolders of target file")?;
    println!(
        "Cutting song: {:.2}+{:.2}: {} to {}",
        info.start_time.time,
        difference,
        info.song,
        target_file.to_str().unwrap()
    );
    let mut command = Command::new("ffmpeg");
    command
        .arg("-ss")
        .arg(format!("{}", info.start_time.time))
        .arg("-t")
        .arg(format!("{}", difference))
        .arg("-i")
        .arg(&info.buffer_file.to_str().unwrap())
        .arg("-c:a")
        .arg("libopus")
        .arg("-b:a")
        .arg(format!("{}", config::BITRATE));
    add_metadata_arg_if_present(
        &mut command,
        |title| format!("title={}", title),
        info.song.title.as_ref(),
    );
    add_metadata_arg_if_present(
        &mut command,
        |album| format!("album={}", album),
        info.song.album.as_ref(),
    );
    add_metadata_arg_if_present(
        &mut command,
        |artist| format!("artist={}", artist),
        info.song.artist.as_ref(),
    );
    add_metadata_arg_if_present(
        &mut command,
        |artist| format!("albumartist={}", artist),
        info.song.artist.as_ref(),
    );
    add_metadata_arg_if_present(
        &mut command,
        |track_number| format!("track={}", track_number),
        info.song.track_number.as_ref(),
    );
    let out = command
        .arg("-y")
        .arg(target_file.to_str().unwrap())
        .output();
    out.map(|_| ()).context(format!(
        "Failed to cut song: {:?} {:?} {:?} ({:?}+{:?}) (is ffmpeg installed?)",
        &info.song.title, &info.song.album, &info.song.artist, info.start_time.time, difference,
    ))
}
