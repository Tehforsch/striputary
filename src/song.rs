use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub struct Song {
    pub artist: String,
    pub album: String,
    pub title: String,
    pub track_number: i64,
    pub length: u64,
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
