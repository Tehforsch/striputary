use crate::config::NUM_SAMPLES_PER_AVERAGE_VOLUME;

pub struct AudioExcerpt {
    pub samples: Vec<f64>,
    pub start_time: f64,
    pub end_time: f64,
    pub delta_t: f64,
}

impl AudioExcerpt {
    pub fn get_volume_at(&self, time: f64) -> f64 {
        let exact_position = ((time - self.start_time) / self.delta_t) as usize;
        let position_begin = 0.max(exact_position - NUM_SAMPLES_PER_AVERAGE_VOLUME);
        let position_end = self.samples.len().max(exact_position - NUM_SAMPLES_PER_AVERAGE_VOLUME);
        let average: f64 = self.samples[position_begin..position_end].iter().sum::<f64>() / ((position_end-position_begin) as f64);
        average
    }
}
