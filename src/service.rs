use std::fmt::Display;
use std::str::FromStr;

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

use crate::recording::dbus::get_instance_of_service;

#[derive(Copy, Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub enum Service {
    #[default]
    SpotifyNative,
    SpotifyChromium,
}

impl FromStr for Service {
    type Err = serde_yaml::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // This is quite ugly, but ensures that the config file string representation
        // is the same as in the command line options (which uses the FromStr impl),
        // without adding any additional dependencies
        serde_yaml::from_str(s)
    }
}

impl Display for Service {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Similar to the from_str implementation, this is ugly but consistent.
        let s = serde_yaml::to_string(self).unwrap();
        let s = s.trim();
        write!(f, "{}", s)
    }
}

impl Service {
    pub fn sink_name(&self) -> &str {
        match self {
            Self::SpotifyNative => "Spotify",
            Self::SpotifyChromium => "Playback",
        }
    }

    pub fn dbus_bus_name(&self) -> String {
        match self {
            Self::SpotifyNative => "org.mpris.MediaPlayer2.spotify".into(),
            Self::SpotifyChromium => get_instance_of_service("org.mpris.MediaPlayer2.chromium")
                .expect("Failed to get dbus service instance"),
        }
    }
}
