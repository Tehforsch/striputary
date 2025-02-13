mod audio;
mod config;
mod consts;
mod gui;
mod recording;
mod recording_session;
mod session_manager;
mod song;

use std::path::Path;

use anyhow::Result;
use config::Command;
use config::Config;
use log::info;
use log::LevelFilter;
use recording::dbus::DbusConnection;
use recording::recorder::Recorder;
use recording_session::SessionPath;
use simplelog::ColorChoice;
use simplelog::ConfigBuilder;
use simplelog::LevelPadding;
use simplelog::TermLogger;
use simplelog::TerminalMode;
use time::UtcOffset;

use crate::gui::Gui;
use crate::recording_session::get_new_name;

fn record(config: &Config) -> Result<()> {
    info!("Using service: {}", config.service);
    info!("Using sound server: {}", config.sound_server);
    let path = SessionPath(get_new_name(&config.output_dir));
    let recorder = Recorder::new(&config, &path)?;
    let _session = recorder.record_new_session()?;
    Ok(())
}

fn monitor_dbus(config: &Config) {
    let conn = DbusConnection::new(&config.service);
    loop {
        for ev in conn.get_new_events() {
            println!("{:?}", ev);
        }
    }
}

fn run_gui(config: &Config) {
    Gui::start(config);
}

fn cut(_: &Config, _: &Path) {
    todo!()
}

fn init_logging(verbosity: usize) {
    let level = match verbosity {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        2 => LevelFilter::Trace,
        v => unimplemented!("Unsupported verbosity level: {}", v),
    };

    let local = chrono::Local::now();
    let offset = local.offset();
    let config = ConfigBuilder::default()
        .set_level_padding(LevelPadding::Right)
        .set_time_offset(UtcOffset::from_whole_seconds(offset.local_minus_utc()).unwrap())
        .build();
    TermLogger::init(level, config, TerminalMode::Mixed, ColorChoice::Auto).unwrap();
}

fn main() -> Result<()> {
    let config = Config::from_config_and_cli();
    init_logging(config.verbosity as usize);
    match config.command {
        Command::Record => record(&config)?,
        Command::Cut(ref args) => cut(&config, &args.path),
        Command::Gui => run_gui(&config),
        Command::MonitorDbus => monitor_dbus(&config),
    }
    Ok(())
}
