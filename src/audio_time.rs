use hound::WavSpec;
use std::ops;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct AudioTime {
    pub time: f64,
    pub interleaved_sample_num: u32,
    pub frame_num: u32,
    pub channels: u16,
    pub sample_rate: u32,
}

impl AudioTime {
    pub fn from_time_and_spec(time: f64, spec: WavSpec) -> AudioTime {
        AudioTime {
            time,
            channels: spec.channels,
            sample_rate: spec.sample_rate,
            interleaved_sample_num: (time * (spec.channels as u32 * spec.sample_rate) as f64)
                as u32,
            frame_num: (time * spec.sample_rate as f64) as u32,
        }
    }
    pub fn from_time_same_spec(time: f64, audiotime: AudioTime) -> AudioTime {
        AudioTime {
            time,
            channels: audiotime.channels,
            sample_rate: audiotime.sample_rate,
            interleaved_sample_num: (time
                * (audiotime.channels as u32 * audiotime.sample_rate) as f64)
                as u32,
            frame_num: (time * audiotime.sample_rate as f64) as u32,
        }
    }
}

impl ops::Sub<AudioTime> for AudioTime {
    type Output = AudioTime;

    fn sub(self, rhs: AudioTime) -> AudioTime {
        assert_eq!(self.sample_rate, rhs.sample_rate);
        assert_eq!(self.channels, rhs.channels);
        AudioTime::from_time_same_spec(self.time - rhs.time, self)
    }
}


impl ops::Add<AudioTime> for AudioTime {
    type Output = AudioTime;

    fn add(self, rhs: AudioTime) -> AudioTime {
        assert_eq!(self.sample_rate, rhs.sample_rate);
        assert_eq!(self.channels, rhs.channels);
        AudioTime::from_time_same_spec(self.time + rhs.time, self)
    }
}

impl PartialOrd for AudioTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.time.partial_cmp(&other.time)
    }
}
