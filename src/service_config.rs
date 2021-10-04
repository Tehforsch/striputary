use anyhow::{anyhow, Result};

#[derive(Clone)]
pub struct ServiceConfig {
    pub sink_name: String,
    pub dbus_bus_name: String,
}

impl ServiceConfig {
    pub fn from_service_name(service_name: &str) -> Result<ServiceConfig> {
        match service_name {
            "spotify" => Ok(ServiceConfig {
                sink_name: "Spotify".to_string(),
                dbus_bus_name: "org.mpris.MediaPlayer2.spotify".to_string(),
            }),
            _ => Err(anyhow!("Unknown service name: {}", service_name)),
        }
    }
}
