#[derive(PartialEq)]
pub enum RecordingStatus {
    FinishedOrInterrupted,
    AlbumFinished,
    NoNewSongForTooLong,
}
