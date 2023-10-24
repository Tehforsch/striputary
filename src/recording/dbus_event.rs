use std::collections::HashMap;

use dbus::arg::PropMap;
use dbus::arg::RefArg;
use dbus::ffidisp::stdintf::org_freedesktop_dbus::PropertiesPropertiesChanged as PC;
use log::error;

use crate::song::Song;

#[derive(Clone, Debug)]
pub enum PlaybackStatus {
    Playing,
    Paused,
}

#[derive(Clone, Debug)]
pub enum DbusEvent {
    NewSong(Song),
    StatusChanged(PlaybackStatus),
}

impl From<PC> for DbusEvent {
    fn from(properties: PC) -> DbusEvent {
        assert!(properties.invalidated_properties.is_empty(), "Invalidated properties not empty, but contains: {:?}. Check if this contains relevant information.", &properties.invalidated_properties);
        let status_changed = get_status_changed(&properties.changed_properties);
        if let Some(status) = status_changed {
            DbusEvent::StatusChanged(status)
        } else {
            DbusEvent::NewSong(
                get_song_from_dbus_properties(properties.changed_properties).unwrap(),
            )
        }
    }
}

fn get_status_changed(properties: &PropMap) -> Option<PlaybackStatus> {
    let has_playback_status_entry = properties.contains_key("PlaybackStatus");
    if has_playback_status_entry {
        let variant = &properties["PlaybackStatus"];
        match variant.0.as_str().unwrap() {
            "Paused" => Some(PlaybackStatus::Paused),
            "Playing" => Some(PlaybackStatus::Playing),
            x => {
                error!("Unknown playback status variant: {}", x);
                None
            }
        }
    } else {
        None
    }
}

fn get_song_from_dbus_properties(properties: PropMap) -> Option<Song> {
    let metadata = &properties.get("Metadata")?.0;

    let mut iter = metadata.as_iter().unwrap();
    let mut dict = Metadata(HashMap::<&str, Box<dyn RefArg>>::new());
    while let Some(key) = iter.next() {
        let value = iter.next().unwrap();
        dict.0.insert(key.as_str().unwrap(), Box::new(value));
    }
    Some(dict.into()).filter(is_valid_song)
}

struct Metadata<'a>(HashMap<&'a str, Box<dyn RefArg + 'a>>);

impl<'a> Metadata<'a> {
    fn get_song_length(&self) -> f64 {
        let val = &self.0["mpris:length"];
        let length_microseconds = val
            .as_u64()
            .or(val.as_i64().map(|x| x as u64))
            .unwrap_or_else(|| {
                val.as_str()
                    .expect("Failed to parse song length as string")
                    .parse()
                    .expect("Failed to parse song length string as integer")
            });
        (length_microseconds as f64) * 1e-6
    }

    fn get_song_artist(&self) -> Option<String> {
        // I want to thank what is probably a combination of spotify and the MediaPlayer2 specification for this wonderful piece of art. Note that spotify doesn't actually send a list of artists, but just the first artist in a nested list which is just great.
        Some(
            self.0["xesam:artist"]
                .as_iter()
                .unwrap()
                .next()
                .unwrap()
                .as_iter()
                .unwrap()
                .next()
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
        )
    }

    fn get_song_album(&self) -> Option<String> {
        self.0
            .get("xesam:album")
            .map(|album| album.as_str().unwrap().to_string())
    }

    fn get_song_title(&self) -> Option<String> {
        Some(self.0["xesam:title"].as_str().unwrap().to_string())
    }

    fn get_song_track_number(&self) -> Option<i64> {
        self.0
            .get("xesam:trackNumber")
            .map(|track_number| track_number.as_i64().unwrap())
    }
}

impl From<Metadata<'_>> for Song {
    fn from(data: Metadata) -> Self {
        Song {
            artist: data.get_song_artist(),

            album: data.get_song_album(),
            title: data.get_song_title(),
            track_number: data.get_song_track_number(),
            length: data.get_song_length(),
        }
    }
}

/// This filters certain malformed entries that
/// some mpris services will send, which contain
/// a track length of zero, and should
/// not be considered actual songs.
fn is_valid_song(song: &Song) -> bool {
    song.length != 0.0
}
