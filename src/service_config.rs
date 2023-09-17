use std::fmt::Display;
use std::str::FromStr;

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

use crate::recording::dbus::get_instance_of_service;

#[derive(Copy, Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Service {
    #[default]
    SpotifyNative,
    SpotifyChromium,
}

impl FromStr for Service {
    type Err = serde_yaml::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // This is quite ugly, but ensures that the config file string representation
        // is the same as in the command line args (which uses the FromStr impl),
        // without adding any additional dependencies
        serde_yaml::from_str(s)
    }
}

impl Display for Service {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Similar to the from_str implementation, this is ugly but consistent.
        write!(f, "{}", serde_yaml::to_string(self).unwrap())
    }
}

#[derive(Clone)]
pub struct ServiceConfig {
    pub sink_name: String,
    pub dbus_bus_name: String,
}

impl ServiceConfig {
    pub fn from_service(service: Service) -> Result<ServiceConfig> {
        use Service::*;

        match service {
            SpotifyNative => Ok(ServiceConfig {
                sink_name: "Spotify".to_string(),
                dbus_bus_name: "org.mpris.MediaPlayer2.spotify".to_string(),
            }),
            SpotifyChromium => Ok(ServiceConfig {
                sink_name: "Playback".to_string(),
                dbus_bus_name: get_instance_of_service("org.mpris.MediaPlayer2.chromium")?,
            }),
        }
    }
}
