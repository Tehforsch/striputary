use std::collections::HashMap;
use std::process::Command;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use dbus::arg::RefArg;
use dbus::ffidisp::stdintf::org_freedesktop_dbus::PropertiesPropertiesChanged as PC;
use dbus::ffidisp::Connection;
use dbus::message::SignalArgs;

use crate::recording::recording_status::RecordingExitStatus;
use crate::recording::recording_status::RecordingStatus;
use crate::recording_session::RecordingSession;
use crate::service_config::ServiceConfig;
use crate::song::Song;

type MetadataDict<'a> = HashMap<&'a str, Box<dyn RefArg + 'a>>;

/// Collect dbus information on the songs.
/// We could collect the dbus timestamps but they are basically useless
/// for cutting the songs since they fluctuate way too much to be precise.
pub fn collect_dbus_info(
    session: &mut RecordingSession,
    service_config: &ServiceConfig,
) -> Result<RecordingStatus> {
    let c = Connection::new_session().unwrap();
    // Add a match for this signal
    let bus_name = service_config.dbus_bus_name.clone();
    let mstr = PC::match_str(Some(&bus_name.into()), None);
    c.add_match(&mstr).unwrap();

    // Wait for the signal to arrive.
    for msg in c.incoming(100) {
        if let Some(pc) = PC::from_message(&msg) {
            return handle_dbus_properties_changed_signal(session, pc);
        }
    }
    Ok(RecordingStatus::Running)
}

pub fn handle_dbus_properties_changed_signal(
    session: &mut RecordingSession,
    properties: PC,
) -> Result<RecordingStatus> {
    let playback_stopped = is_playback_stopped(&properties);
    if !playback_stopped {
        let song = get_song_from_dbus_properties(properties);
        // We get multiple dbus messages on every song change for every property that changes.
        // Find out whether the song actually changed (or whether we havent recorded anything so far)
        let last_song = session.songs.last();
        if let Some(song) = song {
            if session.songs.is_empty() || last_song.unwrap() != &song {
                println!("Now recording song: {}", song);
                session.songs.push(song);
                session.save()?;
            }
        }
    }
    match playback_stopped {
        false => Ok(RecordingStatus::Running),
        true => Ok(RecordingStatus::Finished(
            RecordingExitStatus::FinishedOrInterrupted,
        )),
    }
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

fn get_song_length(metadata: &MetadataDict) -> f64 {
    let val = &metadata["mpris:length"];
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

fn get_song_artist(metadata: &MetadataDict) -> Option<String> {
    // I want to thank what is probably a combination of spotify and the MediaPlayer2 specification for this wonderful piece of art. Note that spotify doesn't actually send a list of artists, but just the first artist in a nested list which is just great.
    Some(
        metadata["xesam:artist"]
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

fn get_song_album(metadata: &MetadataDict) -> Option<String> {
    metadata
        .get("xesam:album")
        .map(|album| album.as_str().unwrap().to_string())
}

fn get_song_title(metadata: &MetadataDict) -> Option<String> {
    Some(metadata["xesam:title"].as_str().unwrap().to_string())
}

fn get_song_track_number(metadata: &MetadataDict) -> Option<i64> {
    metadata
        .get("xesam:trackNumber")
        .map(|track_number| track_number.as_i64().unwrap())
}

/// This filters certain malformed entries that
/// some mpris services will send, which contain
/// a track length of zero, and should
/// not be considered actual songs.
fn is_valid_song(song: &Song) -> bool {
    song.length != 0.0
}

fn get_song_from_dbus_properties(properties: PC) -> Option<Song> {
    let metadata = &properties.changed_properties.get("Metadata")?.0;

    let mut iter = metadata.as_iter().unwrap();
    let mut dict = HashMap::<&str, Box<dyn RefArg>>::new();
    while let Some(key) = iter.next() {
        let value = iter.next().unwrap();
        dict.insert(key.as_str().unwrap(), Box::new(value));
    }
    return Some(Song {
        artist: get_song_artist(&dict),
        album: get_song_album(&dict),
        title: get_song_title(&dict),
        track_number: get_song_track_number(&dict),
        length: get_song_length(&dict),
    })
    .filter(|song| is_valid_song(&song));
}

pub fn dbus_set_playback_status_command(
    service_config: &ServiceConfig,
    command: &str,
) -> Result<()> {
    Command::new("dbus-send")
        .arg("--print-reply")
        .arg(format!("--dest={}", &service_config.dbus_bus_name))
        .arg("/org/mpris/MediaPlayer2")
        .arg(format!("org.mpris.MediaPlayer2.Player.{}", command))
        .output()
        .context("Failed to send dbus command to control playback")
        .map(|_| ()) // We do not need the output, let's not suggest that it is useful for the caller
}

pub fn previous_song(service_config: &ServiceConfig) -> Result<()> {
    dbus_set_playback_status_command(service_config, "Previous")
}

pub fn next_song(service_config: &ServiceConfig) -> Result<()> {
    dbus_set_playback_status_command(service_config, "Next")
}

pub fn start_playback(service_config: &ServiceConfig) -> Result<()> {
    dbus_set_playback_status_command(service_config, "Play")
}

pub fn stop_playback(service_config: &ServiceConfig) -> Result<()> {
    dbus_set_playback_status_command(service_config, "Pause")
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
