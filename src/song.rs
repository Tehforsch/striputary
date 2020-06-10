use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Song {
    pub artist: String,
    pub album: String,
    pub title: String,
    pub track_number: i64,
    pub length: f64,
}

impl Song {
    pub fn get_target_file(&self, music_dir: &Path, i: usize) -> PathBuf {
        let file_name = format!(
            // "{:02}_{}.ogg",
            "{:02}_{:02}_{}.flac",
            i,
            self.track_number,
            &sanitize_string(&self.title)
        );
        music_dir
            .join(Path::new(&sanitize_string(&self.artist)))
            .join(Path::new(&sanitize_string(&self.album)))
            .join(Path::new(&file_name))
    }
}

impl fmt::Display for Song {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({} - {} - {}_{} ({}))",
            self.artist, self.album, self.title, self.track_number, self.length
        )
    }
}

fn sanitize_string(s: &str) -> String {
    s.replace("/", "").replace(" ", "")
}
