use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;

use rodio::OutputStream;
use rodio::Sink;

use crate::audio_excerpt::AudioExcerpt;
use crate::audio_excerpt::AudioExcerptSource;
use crate::audio_time::AudioTime;

pub struct PlaybackThreadHandle {
    shutdown_sender: Sender<ShutdownSignal>,
    start_system_time: SystemTime,
    start_audio_time: AudioTime,
}

impl PlaybackThreadHandle {
    pub fn shut_down(&self) {
        self.shutdown_sender.send(ShutdownSignal).unwrap();
    }

    pub fn get_elapsed_audio_time(&self) -> AudioTime {
        let time_expired = SystemTime::now().duration_since(self.start_system_time);
        let time_expired_secs = time_expired.unwrap_or(Duration::from_millis(0)).as_secs_f64();
        AudioTime::from_time_same_spec(self.start_audio_time.time + time_expired_secs, self.start_audio_time)
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
        if let Ok(_) = shutdown_receiver.recv() {}
    });
    PlaybackThreadHandle { shutdown_sender, start_system_time: SystemTime::now(), start_audio_time: start_time }
}
