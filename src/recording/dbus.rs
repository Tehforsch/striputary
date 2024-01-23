use std::process::Command;
use std::time::Instant;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use dbus::ffidisp::stdintf::org_freedesktop_dbus::PropertiesPropertiesChanged as PC;
use dbus::ffidisp::Connection;
use dbus::message::SignalArgs;
use log::trace;

use super::dbus_event::DbusEvent;
use crate::service::Service;

pub const DBUS_LISTEN_TIMEOUT_MS: u32 = 2;

pub struct DbusConnection {
    service: Service,
    connection: Connection,
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

    pub fn get_new_events<'a>(&'a self) -> impl Iterator<Item = (DbusEvent, Instant)> + 'a {
        self.connection
            .incoming(DBUS_LISTEN_TIMEOUT_MS)
            .filter_map(|msg| {
                let instant = Instant::now();
                trace!("Received dbus msg: {:?}", msg);
                PC::from_message(&msg).map(|pc| (instant, pc))
            })
            .map(move |(instant, pc)| {
                let event = pc.into();
                trace!("Received dbus event: {:?}", event);
                (event, instant)
            })
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
