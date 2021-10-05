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
    pub fn get_target_file(&self, music_dir: &Path) -> PathBuf {
        let file_name = format!(
            "{:02}_{}.opus",
            self.track_number,
            &sanitize_string(&self.title)
        );
        self.get_album_folder(music_dir).join(Path::new(&file_name))
    }

    pub fn get_album_folder(&self, music_dir: &Path) -> PathBuf {
        music_dir
            .join(Path::new(&sanitize_string(&self.artist)))
            .join(Path::new(&sanitize_string(&self.album)))
    }
}

impl fmt::Display for Song {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} - {} - {} ({}s)",
            self.artist, self.album, self.title, self.length.round()
        )
    }
}

fn sanitize_string(s: &str) -> String {
    s.replace("/", "").replace(" ", "")
}
