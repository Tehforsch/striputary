use crate::audio_time::AudioTime;
use crate::config::{NUM_OFFSETS_TO_TRY, NUM_SAMPLES_PER_AVERAGE_VOLUME};
use std::i16;

pub struct AudioExcerpt {
    pub samples: Vec<i16>,
    pub start: AudioTime,
    pub end: AudioTime,
}

impl AudioExcerpt {
    pub fn get_volume_at(&self, time_f64: f64) -> f64 {
        let time = AudioTime::from_time_same_spec(time_f64, self.start);
        let position_exact = time - self.start;
        let position_begin = if position_exact.frame_num < NUM_SAMPLES_PER_AVERAGE_VOLUME as u32 {
            0
        } else {
            position_exact.frame_num as usize - NUM_SAMPLES_PER_AVERAGE_VOLUME
        };
        let position_end = self
            .samples
            .len()
            .min(position_exact.frame_num as usize + NUM_SAMPLES_PER_AVERAGE_VOLUME);
        let inv_len = 1.0 / ((position_end - position_begin) as f64);
        let inv_i16 = 1.0 / (i16::MAX as f64);
        let average: f64 = self.samples[position_begin..position_end]
            .iter()
            .map(|x| (*x as f64).abs() * inv_len * inv_i16)
            .sum::<f64>();
        average
    }

    pub fn get_volume_plot_data(&self) -> Vec<f64> {
        let width = self.end.time - self.start.time;
        let step_size = width / NUM_OFFSETS_TO_TRY as f64;
        let times = (1..NUM_OFFSETS_TO_TRY).map(|x| self.start.time + (x as f64) * step_size);
        times.map(|time| self.get_volume_at(time)).collect()
    }

    pub fn get_time_by_relative_progress(&self, pos: f64) -> f64 {
        self.start.time + (self.end.time - self.start.time) * pos
    }
}
