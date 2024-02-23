use std::collections::HashMap;
use std::time::Duration;

use dbus::arg::PropMap;
use dbus::arg::RefArg;
use dbus::ffidisp::stdintf::org_freedesktop_dbus::PropertiesPropertiesChanged as PC;
use log::error;
use serde::Deserialize;
use serde::Serialize;

use crate::song::Song;

#[derive(Clone, Debug)]
pub enum PlaybackStatus {
    Playing,
    Paused,
}

#[derive(Clone, Debug)]
pub enum PlayerInformation {
    CanGoPrevious(bool),
    CanGoNext(bool),
}

#[derive(Clone, Debug)]
pub enum DbusEvent {
    NewSong(Song),
    NewInvalidSong(Song),
    StatusChanged(PlaybackStatus),
    PlayerInformation(PlayerInformation),
}

#[derive(Serialize, Deserialize, Copy, Debug, Clone)]
pub struct Timestamp {
    pub time_since_start_micros: u128,
}

impl Timestamp {
    pub fn from_duration(timestamp: Duration) -> Timestamp {
        Self {
            time_since_start_micros: timestamp.as_micros(),
        }
    }

    pub fn in_secs(&self) -> f64 {
        self.time_since_start_micros as f64 * 1e-6
    }
}

#[derive(Clone, Debug)]
pub struct TimedDbusEvent {
    pub event: DbusEvent,
    pub timestamp: Timestamp,
}

impl From<PC> for DbusEvent {
    fn from(properties: PC) -> DbusEvent {
        assert!(properties.invalidated_properties.is_empty(), "Invalidated properties not empty, but contains: {:?}. Check if this contains relevant information.", &properties.invalidated_properties);
        let properties = &properties.changed_properties;
        get_status_changed(properties)
            .map(|status| DbusEvent::StatusChanged(status))
            .or(get_player_information(properties).map(|info| DbusEvent::PlayerInformation(info)))
            .unwrap_or_else(|| {
                let (song, is_valid) = get_song_from_dbus_properties(properties);
                if is_valid {
                    DbusEvent::NewSong(song)
                } else {
                    DbusEvent::NewInvalidSong(song)
                }
            })
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

fn get_player_information(properties: &PropMap) -> Option<PlayerInformation> {
    if properties.contains_key("CanGoPrevious") {
        Some(PlayerInformation::CanGoPrevious(
            properties["CanGoPrevious"].as_u64().unwrap() != 0,
        ))
    } else if properties.contains_key("CanGoNext") {
        Some(PlayerInformation::CanGoNext(
            properties["CanGoNext"].as_u64().unwrap() != 0,
        ))
    } else {
        None
    }
}

fn get_song_from_dbus_properties(properties: &PropMap) -> (Song, bool) {
    let metadata = &properties.get("Metadata").unwrap().0;

    let mut iter = metadata.as_iter().unwrap();
    let mut dict = Metadata(HashMap::<&str, Box<dyn RefArg>>::new());
    while let Some(key) = iter.next() {
        let value = iter.next().unwrap();
        dict.0.insert(key.as_str().unwrap(), Box::new(value));
    }
    let song = dict.into();
    let is_valid = is_valid_song(&song);
    (song, is_valid)
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
