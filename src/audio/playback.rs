use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use futures::channel::mpsc::Sender;
use hound::WavSpec;
use rodio::OutputStream;
use rodio::Sink;
use rodio::Source;

use crate::audio::AudioTime;

use super::sample_reader::WavFileReader;
use super::SampleReader;

#[derive(Clone)]
pub struct WavSource {
    spec: WavSpec,
    samples: Vec<i16>,
    position: usize,
}

impl WavSource {
    pub fn new(reader: &mut WavFileReader, start_time: AudioTime, end_time: AudioTime) -> Self {
        let spec = reader.spec();
        let samples = reader.extract_audio(start_time, end_time).unwrap();
        Self {
            spec,
            samples,
            position: 0,
        }
    }
}

impl Source for WavSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.spec.channels
    }

    fn sample_rate(&self) -> u32 {
        self.spec.sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

impl Iterator for WavSource {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.samples.get(self.position);
        self.position += 1;
        item.copied()
    }
}

#[derive(Clone, Debug)]
pub struct Progress {}

pub async fn play_audio(source: WavSource, _: Sender<Progress>) {
    let stop = Arc::new(AtomicBool::new(false));
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    sink.append(source);
    sink.play();
    while !stop.load(Ordering::SeqCst) {
        // async_std::task::sleep(std::time::Duration::from_millis(20)).await;
    }
}
