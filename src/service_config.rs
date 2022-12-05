use anyhow::anyhow;
use anyhow::Result;

use crate::recording::dbus::get_instance_of_service;

#[derive(Clone)]
pub struct ServiceConfig {
    pub sink_name: String,
    pub dbus_bus_name: String,
}

impl ServiceConfig {
    pub fn from_service_name(service_name: &str) -> Result<ServiceConfig> {
        let config = match service_name {
            "spotify" => Ok(ServiceConfig {
                sink_name: "Spotify".to_string(),
                dbus_bus_name: "org.mpris.MediaPlayer2.spotify".to_string(),
            }),
            "spotify-chromium" => Ok(ServiceConfig {
                sink_name: "Playback".to_string(),
                dbus_bus_name: get_instance_of_service("org.mpris.MediaPlayer2.chromium")?,
            }),
            _ => Err(anyhow!("Unknown service name: {}", service_name)),
        };
        config
    }
}
