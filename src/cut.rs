use crate::args::OffsetOpts;
use crate::audio_excerpt::AudioExcerpt;
use crate::config::{MAX_OFFSET, MIN_OFFSET, NUM_OFFSETS_TO_TRY, READ_BUFFER};
use crate::recording_session::RecordingSession;
use crate::song::Song;
use crate::wav::extract_audio;
use itertools::Itertools;
use log::{debug, info};
use std::fs::create_dir_all;
use std::path::Path;
use std::process::Command;
use text_io::{read, try_read, Error};

pub fn group_songs(session: &RecordingSession) -> Vec<(RecordingSession, String)> {
    let mut sessions = vec![];
    if session.songs.len() == 0 {
        return sessions;
    }
    for (song, timestamp) in session.songs.iter().zip(session.timestamps.iter()) {
        if sessions.is_empty() || song.album != sessions.last().unwrap().0.songs[0].album {
            let newSession = RecordingSession {
                dir: session.dir.clone(),
                timestamps: vec![timestamp.clone()],
                songs: vec![song.clone()],
            };
            sessions.push((newSession, song.album.clone()));
        } else {
            sessions.last_mut().unwrap().0.songs.push(song.clone());
        }
    }
    sessions
}

pub fn cut_session(session: RecordingSession, offset_args: &OffsetOpts) {
    for (group, album) in group_songs(&session) {
        info!("Cutting album {}", album);
        cut_group(group, offset_args);
    }
}

pub fn cut_group(group: RecordingSession, offset_args: &OffsetOpts) {
    let cut_timestamps: Vec<f64> = get_cut_timestamps_from_song_lengths(&group);
    let (audio_excerpts, valid_songs) = get_audio_excerpts_and_valid_songs(&group, &cut_timestamps);
    let offset = match offset_args {
        OffsetOpts::Auto => {
            info!("Calculating optimal offset guess");
            determine_cut_offset(audio_excerpts, cut_timestamps)
        }
        OffsetOpts::Manual => get_offset_manually(&group, audio_excerpts),
    };
    info!("Using offset: {:.3}", offset);
    let mut start_time = group.timestamps[0] + offset;
    for (i, song) in valid_songs.iter().enumerate() {
        let end_time = start_time + song.length;
        cut_song(&group, song, start_time, end_time, i);
        start_time = end_time;
    }
    match offset_args {
        OffsetOpts::Manual => {
            if !user_happy_with_offset() {
                cut_group(group, offset_args);
            }
        }
        _ => {}
    }
}

pub fn user_happy_with_offset() -> bool {
    println!("Are the results good? y/n");
    let answer: Result<String, text_io::Error> = try_read!();
    if let Ok(s) = answer {
        s == "y"
    } else {
        false
    }
}

pub fn get_offset_manually(session: &RecordingSession, audio_excerpts: Vec<AudioExcerpt>) -> f64 {
    println!("Enter offset (usually between -2 and 1): ");
    read!()
}

pub fn get_excerpt(buffer_file_name: &Path, cut_time: f64) -> Option<AudioExcerpt> {
    info!("Reading excerpt at {:.2}", cut_time);
    let listen_start_time = cut_time + MIN_OFFSET - READ_BUFFER;
    let listen_end_time = cut_time + MAX_OFFSET + READ_BUFFER;
    extract_audio(buffer_file_name, listen_start_time, listen_end_time).ok()
}

pub fn determine_cut_offset(audio_excerpts: Vec<AudioExcerpt>, cut_timestamps: Vec<f64>) -> f64 {
    // We can assume that some of the songs begin or end with silence.
    // If that is the case then the offset of the cuts should be chosen by finding an offset that
    // puts as many of the cuts at positions where the recording is silent. In other words, the offset is given by
    // the local minimum of convolution of the volume with a sum of dirac deltas at every cut position.
    // In practice I find that this works really well for single albums but the needed offset will increase for
    // songs further into the recording. I think this might be due to some pause that spotify inserts
    // after an album is finished? So for now lets determine the offset for each album individually.
    info!("Listening to excerpts");
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
        // println!("PLOT {} {}", offset, total_volume);
    }
    min.unwrap().1
}

pub fn get_audio_excerpts_and_valid_songs<'lifetime>(
    session: &'lifetime RecordingSession,
    cut_timestamps: &[f64],
) -> (Vec<AudioExcerpt>, Vec<&'lifetime Song>) {
    let mut audio_excerpts = Vec::new();
    let mut valid_songs = Vec::new();

    for (song, cut) in session.songs.iter().zip(cut_timestamps.iter()) {
        let audio_excerpt = get_excerpt(&session.get_buffer_file(), *cut);
        if let Some(excerpt) = audio_excerpt {
            audio_excerpts.push(excerpt);
            valid_songs.push(song);
        } else {
            info!(
                "Found invalid song: {} {} {}",
                &song.artist, &song.album, &song.title
            );
            break;
        }
    }
    (audio_excerpts, valid_songs)
}

pub fn get_cut_timestamps_from_song_lengths(session: &RecordingSession) -> Vec<f64> {
    // let mut cut_timestamps: Vec<f64> = Vec::new();
    session
        .songs
        .iter()
        .scan(session.timestamps[0], |acc, song| {
            *acc += song.length;
            Some(*acc)
        })
        .collect()
}

pub fn cut_song(session: &RecordingSession, song: &Song, start_time: f64, end_time: f64, i: usize) {
    let difference = end_time - start_time;
    let source_file = session.get_buffer_file();
    let target_file = song.get_target_file(&session.get_music_dir(), i);
    create_dir_all(target_file.parent().unwrap())
        .expect("Failed to create subfolders of target file");
    info!(
        "Cutting song: {:.2}+{:.2}: {} to {}",
        start_time,
        difference,
        song,
        target_file.to_str().unwrap()
    );
    let out = Command::new("ffmpeg")
        .arg("-ss")
        .arg(format!("{}", start_time))
        .arg("-t")
        .arg(format!("{}", difference))
        .arg("-i")
        .arg(source_file.to_str().unwrap())
        // .arg("-acodec")
        // .arg("copy")
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
        .arg("-metadata")
        .arg("genre=quarantine")
        .arg("-y")
        .arg(target_file.to_str().unwrap())
        .output()
        .expect("Failed to execute song cutting command");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    debug!("{} {}", stdout, stderr);
}
