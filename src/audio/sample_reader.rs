use std::{fs::File, io::BufReader};

use hound::{WavReader, WavSpec};

use super::time::AudioTime;

pub type WavFileReader = WavReader<BufReader<File>>;

pub trait SampleReader {
    fn extract_audio(
        &mut self,
        start: AudioTime,
        end: AudioTime,
    ) -> Result<Vec<i16>, OutOfBoundsError>;

    fn start(&self) -> AudioTime;
    fn end(&self) -> AudioTime;
    fn spec(&self) -> WavSpec;
}

#[derive(Debug)]
pub struct OutOfBoundsError;

impl From<std::io::Error> for OutOfBoundsError {
    fn from(_error: std::io::Error) -> OutOfBoundsError {
        OutOfBoundsError {}
    }
}

pub fn get_volume_at<R: SampleReader>(r: &mut R, time: AudioTime) -> Result<f64, OutOfBoundsError> {
    pub static NUM_SAMPLES_PER_AVERAGE_VOLUME: usize = 4000;
    let start = time
        - AudioTime::from_sample_and_spec((NUM_SAMPLES_PER_AVERAGE_VOLUME / 2) as u32, r.spec());
    let end = time
        + AudioTime::from_sample_and_spec((NUM_SAMPLES_PER_AVERAGE_VOLUME / 2) as u32, r.spec());
    if end.interleaved_sample_num == start.interleaved_sample_num {
        // This can happen if we underflow to 0 or negative positions.
        // Zero volume is the correct assumption for these positions
        return Ok(0.0);
    }
    let inv_len = 1.0 / ((end.interleaved_sample_num - start.interleaved_sample_num) as f64);
    let inv_i16 = 1.0 / (i16::MAX as f64);
    let samples = r.extract_audio(start, end)?;
    let average: f64 = samples
        .iter()
        .map(|x| (*x as f64).abs() * inv_len * inv_i16)
        .sum::<f64>();
    Ok(average)
}

impl SampleReader for WavReader<BufReader<File>> {
    fn extract_audio(
        &mut self,
        start: AudioTime,
        end: AudioTime,
    ) -> Result<Vec<i16>, OutOfBoundsError> {
        let num_samples = (end - start).interleaved_sample_num;
        self.seek(start.frame_num())?;
        let samples_interleaved: Vec<i16> = self
            .samples::<i16>()
            .take(num_samples as usize)
            .collect::<Result<Vec<_>, hound::Error>>()
            .map_err(|_| OutOfBoundsError)?;
        if samples_interleaved.len() as u32 != num_samples {
            Err(OutOfBoundsError {})
        } else {
            Ok(samples_interleaved)
        }
    }

    fn start(&self) -> AudioTime {
        AudioTime::from_time_and_spec(0.0, self.spec())
    }

    fn end(&self) -> AudioTime {
        AudioTime::from_sample_and_spec(self.len(), self.spec())
    }

    fn spec(&self) -> WavSpec {
        self.spec()
    }
}
