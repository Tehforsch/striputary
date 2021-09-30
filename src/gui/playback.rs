use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;
use std::thread;

use rodio::OutputStream;
use rodio::Sink;

use crate::audio_excerpt::AudioExcerptSource;
use crate::audio_time::AudioTime;
use crate::audio_excerpt::AudioExcerpt;

pub struct PlaybackThreadHandle {
    shutdown_sender: Sender<ShutdownSignal>,
}

impl PlaybackThreadHandle {
    pub fn shut_down(&self) {
        self.shutdown_sender.send(ShutdownSignal).unwrap();
    }
}

pub struct ShutdownSignal;

pub fn play_excerpt(excerpt: &AudioExcerpt, start_time: AudioTime) -> PlaybackThreadHandle {
    let cloned = excerpt.clone();
    let (shutdown_sender, shutdown_receiver) = channel();
    thread::spawn(move || {
        let source = AudioExcerptSource::new(cloned, start_time);
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        sink.append(source);
        sink.play();
        if let Ok(_) = shutdown_receiver.recv() {
        }
    });
    PlaybackThreadHandle {
        shutdown_sender
    }
}
