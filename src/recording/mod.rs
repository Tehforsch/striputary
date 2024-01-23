pub mod audio_recorder;
pub mod dbus;
pub mod dbus_event;
pub mod recorder;
pub use audio_recorder::AudioRecorder;
pub use audio_recorder::SoundServer;

#[derive(PartialEq)]
pub enum RecordingStatus {
    FinishedOrInterrupted,
    AlbumFinished,
    NoNewSongForTooLong,
}

pub enum RecorderStatus {
    Running,
    Failed(anyhow::Error),
    Stopped,
}
