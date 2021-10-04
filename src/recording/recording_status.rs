#[derive(PartialEq)]
pub enum RecordingExitStatus {
    FinishedOrInterrupted,
    AlbumFinished,
}

#[derive(PartialEq)]
pub enum RecordingStatus {
    Running,
    Finished(RecordingExitStatus),
}
