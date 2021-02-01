use crate::audio_excerpt::AudioExcerpt;
use crate::config::NUM_SAMPLES_PER_AVERAGE;
use lewton::inside_ogg::OggStreamReader;
use lewton::VorbisError;
use std::collections::VecDeque;
use std::fs::File;
use std::path::Path;

pub fn average(values: &VecDeque<f64>) -> f64 {
    let mut av: f64 = 0.0;
    let inv_length: f64 = 1.0 / values.len() as f64;
    for v in values {
        av += ((*v) * inv_length).abs();
    }
    av
}

pub fn get_package(srr: &mut OggStreamReader<File>) -> Option<Vec<Vec<i16>>> {
    let pck_read = srr.read_dec_packet();
    match pck_read {
        Ok(pck) => pck,
        Err(_) => {
            // The error means the file ended unexpectedly which happens a lot when
            // recording ogg in the way we do. Just return None so the loop ends
            None
        }
    }
}

pub fn get_audio_excerpt(
    file_path: &Path,
    start_time: f64,
    end_time: f64,
) -> Result<AudioExcerpt, VorbisError> {
    let f = File::open(&file_path).expect("Can't open file");
    let mut srr = OggStreamReader::new(f)?;

    let mut samples: Vec<f64> = Vec::new();
    // let mut timestamps: Vec<f64> = Vec::new();
    let sample_rate = srr.ident_hdr.audio_sample_rate as f64;
    let time_per_average = NUM_SAMPLES_PER_AVERAGE as f64 / sample_rate;
    let num_channels = srr.ident_hdr.audio_channels as usize;
    let mut total_sample_queue: VecDeque<f64> = VecDeque::new();
    let inv_max_i16 = 1.0 / (std::i16::MAX as f64);
    let theoretical_start_sample = (start_time * sample_rate) as u64;
    let theoretical_end_sample = (end_time * sample_rate) as u64;
    srr.seek_absgp_pg(theoretical_start_sample)?;
    let mut first_sample: Option<u64> = None;

    while let Some(packet) = srr.read_dec_packet()? {
        let absgp = srr.get_last_absgp();
        if let Some(t) = absgp {
            if first_sample == None {
                first_sample = Some(t);
            }
            if t > theoretical_end_sample {
                break;
            }
        } else {
            continue;
        }
        for (i, _) in packet[0].iter().enumerate() {
            let mut total: f64 = 0.0;
            for (j, _) in packet.iter().enumerate() {
                total += packet[j][i] as f64 * inv_max_i16;
            }
            total_sample_queue.push_back(total);
            if total_sample_queue.len() == NUM_SAMPLES_PER_AVERAGE {
                samples.push(average(&total_sample_queue) / (num_channels as f64));
                total_sample_queue.clear()
            }
        }
    }
    let last_sample = srr.get_last_absgp().unwrap();
    let audio_excerpt = AudioExcerpt {
        samples: samples,
        start_time: first_sample.unwrap() as f64 / sample_rate,
        end_time: last_sample as f64 / sample_rate,
        delta_t: time_per_average,
    };
    debug!(
        "scanned\n{} {}\n{} {}",
        start_time, end_time, audio_excerpt.start_time, audio_excerpt.end_time
    );
    Ok(audio_excerpt)
}
