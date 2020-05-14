pub struct AudioExcerpt {
    pub samples: Vec<f64>,
    pub start_time: f64,
    pub end_time: f64,
    pub delta_t: f64,
}

impl AudioExcerpt {
    pub fn get_volume_at(&self, time: f64) -> f64 {
        let exact_position = ((time - self.start_time) / self.delta_t) as usize;
        self.samples[exact_position]
    }
}
