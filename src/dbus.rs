use crate::recording_session::RecordingSession;
use crate::song::Song;
use anyhow::{Context, Result};
use dbus::arg::RefArg;
use dbus::ffidisp::stdintf::org_freedesktop_dbus::PropertiesPropertiesChanged as PC;
use dbus::ffidisp::Connection;
use dbus::message::SignalArgs;
use std::collections::HashMap;
use std::process::Command;
use std::time::Instant;

pub fn collect_dbus_timestamps(
    record_start_time: &Instant,
    session: &mut RecordingSession,
) -> bool {
    let c = Connection::new_session().unwrap();
    // Add a match for this signal
    let mstr = PC::match_str(Some(&"org.mpris.MediaPlayer2.spotify".into()), None);
    c.add_match(&mstr).unwrap();

    // Wait for the signal to arrive.
    for msg in c.incoming(100) {
        if let Some(pc) = PC::from_message(&msg) {
            let playback_stopped =
                handle_dbus_properties_changed_signal(record_start_time, session, pc);
            return playback_stopped;
        }
    }
    false
}

pub fn handle_dbus_properties_changed_signal(
    record_start_time: &Instant,
    session: &mut RecordingSession,
    properties: PC,
) -> bool {
    let playback_stopped = is_playback_stopped(&properties);
    if !playback_stopped {
        let song = get_song_from_dbus_properties(properties);
        // We get multiple dbus messages on every song change for every property that changes.
        // Find out whether the song actually changed (or whether we havent recorded anything so far)
        if session.songs.is_empty() || session.songs.last().unwrap() != &song {
            println!("Recording song: {:?}", song);
            session.songs.push(song);
            session
                .timestamps
                .push(record_start_time.elapsed().as_secs_f64());
        }
    }
    playback_stopped
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

#[allow(clippy::needless_return)] // This return is actually not needless.
fn get_song_from_dbus_properties(properties: PC) -> Song {
    let metadata = &properties.changed_properties["Metadata"].0;

    let mut iter = metadata.as_iter().unwrap();
    let mut dict = HashMap::<&str, Box<dyn RefArg>>::new();
    while let Some(key) = iter.next() {
        let value = iter.next().unwrap();
        dict.insert(key.as_str().unwrap(), Box::new(value));
    }

    return Song {
        // I want to thank either spotify or the MediaPlayer2 specification for this wonderful piece of art. Note that spotify doesn't actually send a list of artists, but just the first artist in a nested list which is great.
        artist: dict["xesam:artist"]
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
        album: dict["xesam:album"].as_str().unwrap().to_string(),
        title: dict["xesam:title"].as_str().unwrap().to_string(),
        track_number: dict["xesam:trackNumber"].as_i64().unwrap(),
        length: (dict["mpris:length"].as_u64().unwrap() as f64) * 1e-6, // convert s -> µs
    };
}

pub fn dbus_set_playback_status_command(command: &str) -> Result<()> {
    Command::new("dbus-send")
        .arg("--print-reply")
        .arg("--dest=org.mpris.MediaPlayer2.spotify")
        .arg("/org/mpris/MediaPlayer2")
        .arg(format!("org.mpris.MediaPlayer2.Player.{}", command))
        .output()
        .context("Failed to send dbus command to control playback")
        .map(|_| ()) // We do not need the output, let's not suggest that it is useful for the caller
}

pub fn previous_song() -> Result<()> {
    dbus_set_playback_status_command("Previous")
}

pub fn start_playback() -> Result<()> {
    dbus_set_playback_status_command("Play")
}

pub fn stop_playback() -> Result<()> {
    dbus_set_playback_status_command("Pause")
}
