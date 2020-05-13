use lewton::inside_ogg::OggStreamReader;
use lewton::VorbisError;
use std::fs::File;
use std::path::Path;

pub fn average(values: &Vec<i16>) -> f32 {
    let mut av: f32 = 0.0;
    let inv_length: f32 = 1.0 / values.len() as f32;
    let inv_max_i16 = 1.0 / (std::i16::MAX as f32);
    for v in values {
        av += (*v as f32) * inv_length * inv_max_i16;
    }
    av
}

pub fn get_volume(packet: Vec<Vec<i16>>) -> f32 {
    return (average(&packet[0]) + average(&packet[1])) * 0.5;
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

pub fn get_buffer_file_volume_over_time(
    file_path: &Path,
) -> Result<(Vec<f32>, Vec<f32>), VorbisError> {
    let f = File::open(&file_path).expect("Can't open file");
    let mut srr = OggStreamReader::new(f)?;

    let mut decoded_length = 0.0;
    let mut volume: Vec<f32> = Vec::new();
    let mut timestamps: Vec<f32> = Vec::new();
    while let Some(packet) = get_package(&mut srr) {
        decoded_length += packet[0].len() as f32 / srr.ident_hdr.audio_sample_rate as f32;
        timestamps.push(decoded_length);
        volume.push(get_volume(packet));
    }
    Ok((timestamps, volume))
}
