use crate::recording_session::RecordingSession;
use dbus::arg::RefArg;
use dbus::ffidisp::stdintf::org_freedesktop_dbus::PropertiesPropertiesChanged as PC;
use dbus::ffidisp::Connection;
use dbus::message::SignalArgs;

use log::info;

use std::collections::HashMap;
use std::time::Instant;

use crate::song::Song;

pub fn collect_dbus_timestamps(record_start_time: &Instant, session: &mut RecordingSession) {
    let c = Connection::new_session().unwrap();
    // Add a match for this signal
    let mstr = PC::match_str(Some(&"org.mpris.MediaPlayer2.spotify".into()), None);
    c.add_match(&mstr).unwrap();

    // Wait for the signal to arrive.
    for msg in c.incoming(100) {
        if let Some(pc) = PC::from_message(&msg) {
            handle_dbus_properties_changed_signal(record_start_time, session, pc);
        }
    }
}

pub fn handle_dbus_properties_changed_signal(
    record_start_time: &Instant,
    session: &mut RecordingSession,
    properties: PC,
) {
    let song = get_song_from_dbus_properties(properties);
    // We get multiple dbus messages on every song change for every property that changes.
    // Find out whether the song actually changed (or whether we havent recorded anything so far)
    if session.songs.len() == 0 || session.songs.last().unwrap() != &song {
        info!("Recording song: {:?}", song);
        session.songs.push(song);
        session
            .timestamps
            .push(record_start_time.elapsed().as_millis());
    }
}

fn get_song_from_dbus_properties(properties: PC) -> Song {
    let metadata = &properties.changed_properties["Metadata"].0;

    let mut iter = metadata.as_iter().unwrap();
    let mut dict = HashMap::<&str, Box<dyn RefArg>>::new();
    while let Some(key) = iter.next() {
        let value = iter.next().unwrap();
        dict.insert(key.as_str().unwrap(), Box::new(value));
    }

    let song = Song {
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
        length: dict["mpris:length"].as_u64().unwrap(),
    };
    song
}
