use anyhow::{Context, Result};
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::audio_excerpt::AudioExcerpt;
use crate::config::{MAX_OFFSET, MIN_OFFSET, NUM_OFFSETS_TO_TRY, READ_BUFFER};
use crate::excerpt_collection::{ExcerptCollection, NamedExcerpt};
use crate::recording_session::RecordingSession;
use crate::song::Song;
use crate::wav::extract_audio;

pub struct CutInfo {
    pub song: Song,
    buffer_file: PathBuf,
    music_dir: PathBuf,
    start_time: f64,
    end_time: f64,
}

impl CutInfo {
    pub fn new(session: &RecordingSession, song: Song, start_time: f64, end_time: f64) -> Self {
        let buffer_file = session.get_buffer_file();
        let music_dir = session.get_music_dir();
        CutInfo {
            buffer_file,
            music_dir,
            song,
            start_time,
            end_time,
        }
    }
}

fn get_excerpt(buffer_file_name: &Path, cut_time: f64) -> Option<AudioExcerpt> {
    let listen_start_time = cut_time + MIN_OFFSET - READ_BUFFER;
    let listen_end_time = cut_time + MAX_OFFSET + READ_BUFFER;
    extract_audio(buffer_file_name, listen_start_time, listen_end_time).ok()
}

fn get_cut_timestamps_from_song_lengths(songs: &[Song], estimated_time_first_song: f64) -> Vec<f64> {
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
            .map(|(cut_time, audio_excerpt)| {
                audio_excerpt.get_volume_at(cut_time + offset)
            })
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
    let timestamps = get_cut_timestamps_from_song_lengths(&songs, session.estimated_time_first_song);
    let offset_guess = determine_cut_offset(&excerpts, &timestamps);
    ExcerptCollection {
        session: session,
        excerpts: excerpts
        .into_iter()
        .enumerate()
        .zip(songs.into_iter())
        .map(|((num, excerpt), song)| NamedExcerpt { excerpt, song, num })
        .collect(),
        offset_guess
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
    (audio_excerpts, valid_songs)
}

pub fn cut_song(info: &CutInfo) -> Result<()> {
    let difference = info.end_time - info.start_time;
    let target_file = info.song.get_target_file(&info.music_dir);
    create_dir_all(target_file.parent().unwrap())
        .context("Failed to create subfolders of target file")?;
    println!(
        "Cutting song: {:.2}+{:.2}: {} to {}",
        info.start_time,
        difference,
        info.song,
        target_file.to_str().unwrap()
    );
    Command::new("ffmpeg")
        .arg("-ss")
        .arg(format!("{}", info.start_time))
        .arg("-t")
        .arg(format!("{}", difference))
        .arg("-i")
        .arg(&info.buffer_file.to_str().unwrap())
        .arg("-metadata")
        .arg(format!("title={}", &info.song.title))
        .arg("-metadata")
        .arg(format!("album={}", &info.song.album))
        .arg("-metadata")
        .arg(format!("artist={}", &info.song.artist))
        .arg("-metadata")
        .arg(format!("albumartist={}", &info.song.artist))
        .arg("-metadata")
        .arg(format!("track={}", &info.song.track_number))
        .arg("-y")
        .arg(target_file.to_str().unwrap())
        .output()
        .map(|_| ())
        .context(format!(
            "Failed to cut song: {} {} {} ({}+{})",
            &info.song.title, &info.song.album, &info.song.artist, info.start_time, difference,
        ))
}
