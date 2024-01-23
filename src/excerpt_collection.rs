use crate::audio_excerpt::AudioExcerpt;
use crate::recording_session::RecordingSessionWithPath;
use crate::song::Song;

#[derive(Clone)]
pub struct NamedExcerpt {
    pub excerpt: AudioExcerpt,
    pub song_before: Option<Song>,
    pub song_after: Option<Song>,
    pub num: usize,
}

#[derive(Clone)]
pub struct ExcerptCollection {
    pub session: RecordingSessionWithPath,
    pub excerpts: Vec<NamedExcerpt>,
    pub offset_guess: f64,
}
