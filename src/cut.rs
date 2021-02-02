use anyhow::{anyhow, Context, Result};
use std::fs::create_dir_all;
use std::path::Path;
use std::process::Command;
use text_io::{read, try_read};

use crate::args::{CutOpts, OffsetOpts, OffsetPosition};
use crate::audio_excerpt::AudioExcerpt;
use crate::config::{MAX_OFFSET, MIN_OFFSET, NUM_OFFSETS_TO_TRY, READ_BUFFER};
use crate::recording_session::RecordingSession;
use crate::song::Song;
use crate::wav::extract_audio;

pub fn cut_session(session: RecordingSession, cut_args: &CutOpts) -> Result<()> {
    // In practice I find that determining the offset works really well for single albums but the needed offset
    // will increase for songs further into the recording. I think this might be due to some pause that is
    // inserted after an album is finished? So for now lets determine the offset for each album individually.
    // print_timestamps_vs_song_lengths(&session);
    for (group, album_title) in group_songs_by_album(&session).iter() {
        println!("Cutting album {}", album_title);
        cut_group(group, cut_args)?;
    }
    Ok(())
}

fn print_timestamps_vs_song_lengths(session: &RecordingSession) -> () {
    let mut acc_length = 0.0;
    let initial_timestamp = session.timestamps[0];
    for (song, timestamp) in session.songs.iter().zip(session.timestamps.iter()) {
        println!(
            "{:.2} {:.2} {:.2}",
            (acc_length - (timestamp - initial_timestamp)),
            acc_length,
            timestamp - initial_timestamp
        );
        acc_length += song.length;
    }
    todo!()
}

pub fn group_songs_by_album(session: &RecordingSession) -> Vec<(RecordingSession, String)> {
    let mut sessions = vec![];
    if session.songs.len() == 0 {
        return sessions;
    }
    for (song, timestamp) in session.songs.iter().zip(session.timestamps.iter()) {
        if sessions.is_empty() || song.album != sessions.last().unwrap().0.songs[0].album {
            let new_session = RecordingSession {
                dir: session.dir.clone(),
                timestamps: vec![timestamp.clone()],
                songs: vec![song.clone()],
            };
            sessions.push((new_session, song.album.clone()));
        } else {
            sessions.last_mut().unwrap().0.songs.push(song.clone());
        }
    }
    sessions
}

pub fn cut_group(group: &RecordingSession, cut_args: &CutOpts) -> Result<()> {
    let cut_timestamps: Vec<f64> = get_cut_timestamps_from_song_lengths(group);
    let (audio_excerpts, valid_songs) = get_audio_excerpts_and_valid_songs(group, &cut_timestamps)?;
    let offset = match &cut_args.offset {
        OffsetOpts::Auto => {
            println!("Calculating ideal offset");
            determine_cut_offset(audio_excerpts, cut_timestamps)
        }
        OffsetOpts::Manual(off) => off.position,
    };
    println!("Using offset: {:.3}", offset);
    let mut start_time = group.timestamps[0] + offset;
    for song in valid_songs.iter() {
        let end_time = start_time + song.length;
        cut_song(group, song, start_time, end_time)?;
        start_time = end_time;
    }
    if !user_happy_with_offset(group)? {
        cut_group(group, &get_manual_cut_options())?;
    }
    Ok(())
}

fn get_manual_cut_options() -> CutOpts {
    CutOpts {
        offset: OffsetOpts::Manual(OffsetPosition {
            position: get_offset_interactively(),
        }),
    }
}

fn user_happy_with_offset(session: &RecordingSession) -> Result<bool> {
    playback_session(session)?;
    println!("Are the results good? y/N");
    let answer: Result<String, text_io::Error> = try_read!();
    if let Ok(s) = answer {
        Ok(s == "y")
    } else {
        Ok(false)
    }
}

fn playback_session(session: &RecordingSession) -> Result<()> {
    let album_folder = session.songs[0].get_album_folder(&session.get_music_dir());
    Command::new("vlc")
        .arg(album_folder.to_str().unwrap())
        .output()
        .map(|_| ())
        .context("Failed to run playback program")
}

pub fn get_offset_interactively() -> f64 {
    println!("Enter offset (usually between -2 and 1): ");
    read!()
}

pub fn get_excerpt(buffer_file_name: &Path, cut_time: f64) -> Option<AudioExcerpt> {
    let listen_start_time = cut_time + MIN_OFFSET - READ_BUFFER;
    let listen_end_time = cut_time + MAX_OFFSET + READ_BUFFER;
    extract_audio(buffer_file_name, listen_start_time, listen_end_time).ok()
}

pub fn determine_cut_offset(audio_excerpts: Vec<AudioExcerpt>, cut_timestamps: Vec<f64>) -> f64 {
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
    min.unwrap().1
}

pub fn get_audio_excerpts_and_valid_songs<'a>(
    session: &'a RecordingSession,
    cut_timestamps: &[f64],
) -> Result<(Vec<AudioExcerpt>, Vec<&'a Song>)> {
    let mut audio_excerpts = Vec::new();
    let mut valid_songs = Vec::new();

    for (song, cut) in session.songs.iter().zip(cut_timestamps.iter()) {
        let audio_excerpt = get_excerpt(&session.get_buffer_file(), *cut);
        if let Some(excerpt) = audio_excerpt {
            audio_excerpts.push(excerpt);
            valid_songs.push(song);
        } else {
            return Err(anyhow!(
                "Found invalid song: {} {} {}",
                &song.artist,
                &song.album,
                &song.title
            ));
        }
    }
    Ok((audio_excerpts, valid_songs))
}

pub fn get_cut_timestamps_from_song_lengths(session: &RecordingSession) -> Vec<f64> {
    session
        .songs
        .iter()
        .scan(session.timestamps[0], |acc, song| {
            *acc += song.length;
            Some(*acc)
        })
        .collect()
}

pub fn cut_song(
    session: &RecordingSession,
    song: &Song,
    start_time: f64,
    end_time: f64,
) -> Result<()> {
    let difference = end_time - start_time;
    let source_file = session.get_buffer_file();
    let target_file = song.get_target_file(&session.get_music_dir());
    create_dir_all(target_file.parent().unwrap())
        .context("Failed to create subfolders of target file")?;
    println!(
        "Cutting song: {:.2}+{:.2}: {} to {}",
        start_time,
        difference,
        song,
        target_file.to_str().unwrap()
    );
    Command::new("ffmpeg")
        .arg("-ss")
        .arg(format!("{}", start_time))
        .arg("-t")
        .arg(format!("{}", difference))
        .arg("-i")
        .arg(source_file.to_str().unwrap())
        .arg("-metadata")
        .arg(format!("title={}", &song.title))
        .arg("-metadata")
        .arg(format!("album={}", &song.album))
        .arg("-metadata")
        .arg(format!("artist={}", &song.artist))
        .arg("-metadata")
        .arg(format!("albumartist={}", &song.artist))
        .arg("-metadata")
        .arg(format!("track={}", &song.track_number))
        .arg("-y")
        .arg(target_file.to_str().unwrap())
        .output()
        .map(|_| ())
        .context(format!(
            "Failed to cut song: {} {} {} ({}+{})",
            &song.title, &song.album, &song.artist, start_time, difference,
        ))
}
