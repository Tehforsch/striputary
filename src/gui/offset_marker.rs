use crate::audio_excerpt::AudioExcerpt;

use super::config::{SONG_X_END, SONG_X_START};

pub struct PositionMarker {
    pub num: usize,
    pos: f64,
}

impl PositionMarker {
    pub fn new(num: usize, offset: f64) -> Self {
        Self { num, pos: offset }
    }

    pub fn set_pos_from_world_pos(&mut self, world_pos_x: f32) {
        self.pos = ((world_pos_x - SONG_X_START) / (SONG_X_END - SONG_X_START)) as f64;
    }

    pub fn get_world_pos(&self) -> f32 {
        SONG_X_START + self.pos as f32 * (SONG_X_END - SONG_X_START)
    }

    pub fn get_audio_time(&self, excerpt: &AudioExcerpt) -> f64 {
        excerpt.get_time_by_relative_progress(self.pos)
    }
}
