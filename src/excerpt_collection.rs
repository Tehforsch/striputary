use crate::{audio_excerpt::AudioExcerpt, recording_session::RecordingSession, song::Song};

#[derive(Clone)]
pub struct NamedExcerpt {
    pub excerpt: AudioExcerpt,
    pub song_before: Option<Song>,
    pub song_after: Option<Song>,
    pub num: usize,
}

#[derive(Clone)]
pub struct ExcerptCollection {
    pub session: RecordingSession,
    pub excerpts: Vec<NamedExcerpt>,
    pub offset_guess: f64,
}

impl ExcerptCollection {
    pub fn iter_excerpts<'a>(&'a self) -> Box<dyn Iterator<Item = &NamedExcerpt> + 'a> {
        Box::new(self.excerpts.iter())
    }

    pub fn get_excerpt(&self, num: usize) -> &NamedExcerpt {
        &self.excerpts[num]
    }
}
