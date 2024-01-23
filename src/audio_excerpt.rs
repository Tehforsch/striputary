use std::i16;

use hound::WavSpec;
use rodio::Source;

use crate::audio_time::AudioTime;
use crate::config::NUM_PLOT_DATA_POINTS;
use crate::config::NUM_SAMPLES_PER_AVERAGE_VOLUME;

#[derive(Clone)]
pub struct AudioExcerpt {
    pub samples: Vec<i16>,
    pub start: AudioTime,
    pub end: AudioTime,
    pub spec: WavSpec,
}

impl AudioExcerpt {
    pub fn get_volume_at(&self, time_f64: f64) -> f64 {
        let time = AudioTime::from_time_same_spec(time_f64, self.start);
        let position_exact = time - self.start;
        let position_begin =
            if position_exact.interleaved_sample_num < NUM_SAMPLES_PER_AVERAGE_VOLUME as u32 {
                0
            } else {
                position_exact.interleaved_sample_num as usize - NUM_SAMPLES_PER_AVERAGE_VOLUME
            };
        let position_end = self
            .samples
            .len()
            .min(position_exact.interleaved_sample_num as usize + NUM_SAMPLES_PER_AVERAGE_VOLUME);
        let inv_len = 1.0 / ((position_end - position_begin) as f64);
        let inv_i16 = 1.0 / (i16::MAX as f64);
        let average: f64 = self.samples[position_begin..position_end]
            .iter()
            .map(|x| (*x as f64).abs() * inv_len * inv_i16)
            .sum::<f64>();
        average
    }

    pub fn get_sample_times(&self) -> Vec<f32> {
        let width = self.end.time - self.start.time;
        let step_size = width as f32 / NUM_PLOT_DATA_POINTS as f32;
        (1..NUM_PLOT_DATA_POINTS)
            .map(|x| self.start.time as f32 + (x as f32) * step_size)
            .collect()
    }

    pub fn get_volume_plot_data(&self) -> Vec<f32> {
        let times = self.get_sample_times();
        times
            .into_iter()
            .map(|time| self.get_volume_at(time as f64) as f32)
            .collect()
    }

    pub fn get_absolute_time_by_relative_progress(&self, pos: f64) -> AudioTime {
        AudioTime::from_time_and_spec(
            self.start.time + (self.end.time - self.start.time) * pos,
            self.spec,
        )
    }

    pub fn get_relative_time(&self, absolute_time: AudioTime) -> AudioTime {
        absolute_time - self.start
    }

    pub fn get_relative_progress_from_time_offset(&self, time_offset: f64) -> f64 {
        // time_offset is relative to the center
        0.5 + (time_offset / (self.end.time - self.start.time))
    }

    pub fn get_absolute_time_from_time_offset(&self, time_offset: f64) -> AudioTime {
        self.get_absolute_time_by_relative_progress(
            self.get_relative_progress_from_time_offset(time_offset),
        )
    }
}

pub struct AudioExcerptSource {
    excerpt: AudioExcerpt,
    position: u32,
}

impl AudioExcerptSource {
    pub fn new(excerpt: AudioExcerpt, start_time: AudioTime) -> Self {
        Self {
            excerpt,
            position: start_time.interleaved_sample_num,
        }
    }
}

impl Source for AudioExcerptSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.excerpt.spec.channels
    }

    fn sample_rate(&self) -> u32 {
        self.excerpt.spec.sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

impl Iterator for AudioExcerptSource {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.excerpt.samples.get(self.position as usize);
        self.position += 1;
        item.copied()
    }
}
