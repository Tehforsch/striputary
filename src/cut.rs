use crate::audio_excerpt::AudioExcerpt;
use crate::config::{MAX_OFFSET, MAX_SEEK_ERROR};
use crate::ogg::get_audio_excerpt;
use crate::recording_session::RecordingSession;
use crate::song::Song;
use lewton::VorbisError;
use log::{debug, info};
use std::fs::create_dir_all;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

pub fn cut_session(session: RecordingSession) -> Result<(), VorbisError> {
    cut_session_lengths(session)
}

pub fn determine_cut_offset(
    buffer_file_name: &Path,
    cut_timestamps: Vec<f64>,
) -> Result<f64, VorbisError> {
    // When there are enough songs recorded in the recording session we can assume that some of them begin or end
    // with silence. If that is the case then the offset of all the cuts should be chosen by finding an offset that
    // puts as many of the cuts at positions where the recording is silent. In other words, the offset is given by
    // the local minimum of convolution of the volume with a sum of dirac deltas at every cut position.
    // It might be preferable to choose top hat functions instead of dirac deltas to
    // make the convolution continuous
    let mut audio_samples: Vec<AudioExcerpt> = Vec::new();
    for cut_time in cut_timestamps.iter() {
        let listen_start_time = cut_time - MAX_OFFSET;
        let listen_end_time = cut_time + MAX_OFFSET;
        audio_samples.push(get_audio_excerpt(
            buffer_file_name,
            listen_start_time,
            listen_end_time,
        )?);
    }
    let mut min: Option<(f64, f64)> = None;
    for i in -100..100 {
        let offset = i as f64 * 0.01 * (MAX_OFFSET - MAX_SEEK_ERROR);
        let total_volume: f64 = cut_timestamps
            .iter()
            .zip(audio_samples.iter())
            .map(|(cut_time, audio_sample)| audio_sample.get_volume_at(cut_time + offset))
            .sum();
        if let Some((min_volume, _)) = min {
            if total_volume < min_volume {
                min = Some((total_volume, offset));
            }
        } else {
            min = Some((total_volume, offset));
        };
    }
    Ok(min.unwrap().1)
}

pub fn cut_session_lengths(session: RecordingSession) -> Result<(), VorbisError> {
    let mut cut_timestamps: Vec<f64> = Vec::new();
    cut_timestamps.append(
        &mut session
            .songs
            .iter()
            .scan(session.timestamps[0], |acc, song| {
                *acc += song.length;
                Some(*acc)
            })
            .collect(),
    );
    let offset = determine_cut_offset(&session.get_buffer_file(), cut_timestamps)?;
    // let offset = 0.0;
    info!("Determined offset: {:.3}", offset);
    let mut start_time = session.timestamps[0] + offset;
    for song in session.songs.iter() {
        let end_time = start_time.clone() + song.length;
        cut_song(session.get_buffer_file(), song, start_time, end_time);
        start_time = end_time;
    }
    Ok(())
}

pub fn cut_song(source_file: PathBuf, song: &Song, start_time: f64, end_time: f64) {
    let difference = end_time - start_time;
    let music_dir = Path::new("music");
    let target_file = song.get_target_file(&music_dir);
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
        .arg("-acodec")
        .arg("copy")
        .arg("-y")
        .arg(target_file.to_str().unwrap())
        .output()
        .expect("Failed to execute song cutting command");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    debug!("{} {}", stdout, stderr);
}
