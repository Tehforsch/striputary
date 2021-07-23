use std::thread;

use bevy::prelude::*;
use rodio::OutputStream;
use rodio::Sink;

use crate::audio_excerpt::AudioExcerptSource;
use crate::{audio_excerpt::AudioExcerpt, excerpt_collections::ExcerptCollections};

pub struct PlaybackEvent;

pub fn playback_system(
    collections: Res<ExcerptCollections>,
    mut playback_events: EventReader<PlaybackEvent>,
) {
    for _ in playback_events.iter() {
        let collection = collections.get_selected();
        let excerpt = collection.iter_excerpts().next().unwrap();
        play_excerpt(&excerpt.excerpt);
    }
}

fn play_excerpt(excerpt: &AudioExcerpt) {
    let cloned = excerpt.clone();
    thread::spawn(move || {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        sink.append(AudioExcerptSource::new(cloned));
        sink.sleep_until_end();
    });
}
