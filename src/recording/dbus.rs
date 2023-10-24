use std::collections::HashMap;
use std::process::Command;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use dbus::arg::RefArg;
use dbus::ffidisp::stdintf::org_freedesktop_dbus::PropertiesPropertiesChanged as PC;
use dbus::ffidisp::Connection;
use dbus::message::SignalArgs;

use crate::service::Service;
use crate::song::Song;

struct Metadata<'a>(HashMap<&'a str, Box<dyn RefArg + 'a>>);

pub struct DbusConnection {
    service: Service,
    connection: Connection,
}

pub enum DbusEvent {
    NewSong(Song),
    PlaybackStopped,
}

impl DbusConnection {
    pub fn new(service: &Service) -> Self {
        let connection = Connection::new_session().unwrap();
        // Add a match for this signal
        let bus_name = service.dbus_bus_name();
        let mstr = PC::match_str(Some(&bus_name.into()), None);
        connection.add_match(&mstr).unwrap();
        Self {
            service: service.clone(),
            connection,
        }
    }

    pub fn get_new_events<'a>(&'a self) -> impl Iterator<Item = DbusEvent> + 'a {
        // We could collect the dbus timestamps but they are basically useless
        // for cutting the songs since they fluctuate way too much to be precise.
        self.connection
            .incoming(100)
            .filter_map(|msg| PC::from_message(&msg))
            .filter_map(move |pc| self.handle_dbus_properties_changed_signal(pc))
    }

    pub fn handle_dbus_properties_changed_signal(&self, properties: PC) -> Option<DbusEvent> {
        let playback_stopped = is_playback_stopped(&properties);
        if playback_stopped {
            Some(DbusEvent::PlaybackStopped)
        } else {
            let song = get_song_from_dbus_properties(properties);
            if let Some(song) = song {
                Some(DbusEvent::NewSong(song))
            } else {
                None
            }
        }
    }

    pub fn previous_song(&self) -> Result<()> {
        dbus_set_playback_status_command(&self.service, "Previous")
    }

    pub fn next_song(&self) -> Result<()> {
        dbus_set_playback_status_command(&self.service, "Next")
    }

    pub fn start_playback(&self) -> Result<()> {
        dbus_set_playback_status_command(&self.service, "Play")
    }

    pub fn stop_playback(&self) -> Result<()> {
        dbus_set_playback_status_command(&self.service, "Pause")
    }
}

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

fn is_playback_stopped(properties: &PC) -> bool {
    let has_playback_status_entry = properties.changed_properties.contains_key("PlaybackStatus");
    if has_playback_status_entry {
        let variant = &properties.changed_properties["PlaybackStatus"];
        (variant.0).as_str().unwrap() == "Paused"
    } else {
        false
    }
}

fn get_song_from_dbus_properties(properties: PC) -> Option<Song> {
    let metadata = &properties.changed_properties.get("Metadata")?.0;

    let mut iter = metadata.as_iter().unwrap();
    let mut dict = Metadata(HashMap::<&str, Box<dyn RefArg>>::new());
    while let Some(key) = iter.next() {
        let value = iter.next().unwrap();
        dict.0.insert(key.as_str().unwrap(), Box::new(value));
    }
    Some(dict.into()).filter(is_valid_song)
}

pub fn dbus_set_playback_status_command(service: &Service, command: &str) -> Result<()> {
    Command::new("dbus-send")
        .arg("--print-reply")
        .arg(format!("--dest={}", &service.dbus_bus_name()))
        .arg("/org/mpris/MediaPlayer2")
        .arg(format!("org.mpris.MediaPlayer2.Player.{}", command))
        .output()
        .context("Failed to send dbus command to control playback")
        .map(|_| ()) // We do not need the output, let's not suggest that it is useful for the caller
}

/// For some mpris services, the name is not constant
/// but changes depending on the instance id running.
/// Here, we get a list of all available services
/// and find the matching one. Returns an error
/// if there are multiple matches.
pub fn get_instance_of_service(service_base_name: &str) -> Result<String> {
    let out = Command::new("qdbus")
        .arg("--session")
        .output()
        .context("Failed to get list of services with qdbus")?;
    let out = String::from_utf8(out.stdout)?;
    let matching_lines: Vec<_> = out
        .lines()
        .map(|line| line.trim())
        .filter(|line| line.starts_with(service_base_name))
        .collect();
    if matching_lines.len() > 1 {
        Err(anyhow!(
            "Found multiple dbus services that match the service configuration: {}",
            matching_lines.join(", ")
        ))
    } else if matching_lines.is_empty() {
        Err(anyhow!(
            "Found no matching dbus service for base name: {}",
            service_base_name
        ))
    } else {
        Ok(matching_lines[0].into())
    }
}
