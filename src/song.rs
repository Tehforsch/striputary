use std::fmt;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Song {
    pub artist: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
    pub track_number: Option<i64>,
    pub length: f64,
}

impl Song {
    pub fn get_target_file(&self, music_dir: &Path, num_in_recording: usize) -> PathBuf {
        let track_number_str = if let Some(track_number) = self.track_number {
            format!("{:02}", track_number)
        } else {
            format!("recording_{}", num_in_recording)
        };
        let file_name = format!("{}_{}.opus", track_number_str, format_title(&self.title),);
        self.get_album_folder(music_dir).join(Path::new(&file_name))
    }

    pub fn get_album_folder(&self, music_dir: &Path) -> PathBuf {
        music_dir
            .join(Path::new(&format_artist(&self.artist)))
            .join(Path::new(&format_album(&self.album)))
    }
}

impl fmt::Display for Song {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} - {} - {} ({}s)",
            format_artist(&self.artist),
            format_album(&self.album),
            format_title(&self.title),
            self.length.round()
        )
    }
}

fn sanitize_string(s: &str) -> String {
    let first_item = if s.contains(',') {
        s.split(',').next().unwrap()
    } else {
        s
    };
    first_item.replace(['/', ' '], "")
}

fn sanitize_or_default(s: &Option<String>, default: &str) -> String {
    s.as_ref()
        .map(|s| sanitize_string(s))
        .filter(|s| !s.is_empty())
        .unwrap_or(default.into())
}

pub fn format_title(title: &Option<String>) -> String {
    sanitize_or_default(title, "unknown_title")
}

pub fn format_album(album: &Option<String>) -> String {
    sanitize_or_default(album, "unknown_album")
}

pub fn format_artist(artist: &Option<String>) -> String {
    sanitize_or_default(artist, "unknown_artist")
}
