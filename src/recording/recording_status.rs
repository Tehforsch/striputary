#[derive(PartialEq)]
pub enum RecordingExitStatus {
    FinishedOrInterrupted,
    AlbumFinished,
    NoNewSongForTooLong,
}

#[derive(PartialEq)]
pub enum RecordingStatus {
    Running,
    Finished(RecordingExitStatus),
}
