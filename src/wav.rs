use crate::audio_excerpt::AudioExcerpt;
use crate::audio_time::AudioTime;
use hound;
use std::path::Path;

pub fn get_volume_average_over_channels(samples: Vec<i16>) -> Vec<i16> {
    samples
        .chunks_exact(2)
        .map(|c| (c[0] / 2) + (c[1] / 2))
        .collect()
}

pub fn make_even(number: u32) -> u32 {
    if number % 2 == 0 {
        number
    } else {
        number + 1
    }
}

pub fn get_audio_excerpt(
    file_path: &Path,
    start_time: f64,
    end_time: f64,
) -> Result<AudioExcerpt, hound::Error> {
    let mut reader = hound::WavReader::open(file_path).unwrap();
    let spec = reader.spec();
    let start = AudioTime::from_time_and_spec(start_time, spec);
    let end = AudioTime::from_time_and_spec(end_time, spec);
    let num_samples = (end - start).interleaved_sample_num;
    reader.seek(start.frame_num).expect("Couldn't seek");
    let samples_interleaved: Result<Vec<i16>, hound::Error> =
        reader.samples::<i16>().take(num_samples as usize).collect();
    let samples = get_volume_average_over_channels(samples_interleaved?);
    Ok(AudioExcerpt {
        samples,
        start,
        end,
    })
}
