use std::thread;

use bevy::prelude::*;
use rodio::OutputStream;
use rodio::Sink;

use crate::audio_excerpt::AudioExcerptSource;
use crate::audio_time::AudioTime;
use crate::{audio_excerpt::AudioExcerpt, excerpt_collections::ExcerptCollections};

use super::offset_marker::PositionMarker;
use super::SelectedSong;

pub struct PlaybackEvent;

pub fn playback_system(
    collections: Res<ExcerptCollections>,
    mut playback_events: EventReader<PlaybackEvent>,
    markers: Query<&PositionMarker>,
    selected_song: Res<SelectedSong>,
) {
    for _ in playback_events.iter() {
        let collection = collections.get_selected();
        let excerpt = collection.get_excerpt(selected_song.0);
        let marker = markers
            .iter()
            .find(|marker| marker.num == selected_song.0)
            .unwrap();
        let audio_time = marker.get_relative_time(&excerpt.excerpt);
        play_excerpt(&excerpt.excerpt, audio_time);
    }
}

fn play_excerpt(excerpt: &AudioExcerpt, start_time: AudioTime) {
    let cloned = excerpt.clone();
    thread::spawn(move || {
        let source = AudioExcerptSource::new(cloned, start_time);
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        sink.append(source);
        sink.sleep_until_end();
    });
}
