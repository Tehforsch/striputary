use crate::{audio_excerpt::AudioExcerpt, recording_session::RecordingSession, song::Song};

pub struct NamedExcerpt {
    pub excerpt: AudioExcerpt,
    pub song: Option<Song>,
    pub num: usize,
}

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
