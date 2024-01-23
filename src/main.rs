pub(crate) mod audio_excerpt;
pub(crate) mod audio_time;
pub(crate) mod config;
pub(crate) mod config_file;
pub(crate) mod cut;
pub(crate) mod data_stream;
pub(crate) mod errors;
pub(crate) mod excerpt_collection;
pub(crate) mod gui;
pub(crate) mod recording;
pub(crate) mod recording_session;
pub(crate) mod service;
pub(crate) mod song;
pub(crate) mod wav;

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use config_file::ConfigFile;
use gui::session_manager::get_new_name;
use log::error;
use log::info;
use log::LevelFilter;
use recording::dbus::DbusConnection;
use recording::recorder::Recorder;
use recording::SoundServer;
use recording_session::SessionPath;
use service::Service;
use simplelog::ColorChoice;
use simplelog::ConfigBuilder;
use simplelog::LevelPadding;
use simplelog::TermLogger;
use simplelog::TerminalMode;
use time::UtcOffset;

use crate::gui::StriputaryGui;

#[derive(Parser, Debug, Clone)]
pub enum Command {
    Record,
    Cut,
    MonitorDbus,
}

#[derive(clap::StructOpt, Clone)]
#[clap(version)]
struct CliOpts {
    /// The output directory to record to.
    /// Passing this argument will override the setting
    /// in the config file.
    pub output_dir: Option<PathBuf>,
    /// The service to record.  Passing this argument will override
    /// the setting in the config file.
    service: Option<Service>,
    /// The sound server to use.  Passing this argument will override
    /// the setting in the config file.
    sound_server: Option<SoundServer>,
    #[clap(short, parse(from_occurrences))]
    pub verbosity: usize,
    #[clap(subcommand)]
    command: Command,
}

#[derive(Clone)]
pub struct Opts {
    pub output_dir: PathBuf,
    pub service: Service,
    pub sound_server: SoundServer,
    pub command: Command,
}

impl Opts {
    fn new(opts: CliOpts, config_file: Option<ConfigFile>) -> Opts {
        let service = opts
            .service
            .or(config_file.as_ref().and_then(|file| file.service))
            .unwrap_or_else(|| {
                let service = Service::default();
                info!(
                    "No service specified in command line options or config file. Using default: {:?}.", service
                );
                service
            });
        let sound_server = opts
            .sound_server
            .or(config_file.as_ref().and_then(|file| file.sound_server))
            .unwrap_or_else(|| {
                let service = SoundServer::default();
                info!(
                    "No sound server specified in command line options or config file. Using default: {:?}.", service
                );
                service
            });
        let output_dir = opts
            .output_dir
            .or(config_file.as_ref().map(|file| file.output_dir.clone()))
            .unwrap_or_else(|| {
panic!("Need an output folder - either pass it as a command line argument or specify it in the config file (probably ~/.config/striputary/config.yaml")
            })
            ;
        Opts {
            output_dir,
            service,
            sound_server,
            command: opts.command,
        }
    }
}

fn main() -> Result<()> {
    let opts = CliOpts::parse();
    let config_file = ConfigFile::read();
    if let Err(ref e) = config_file {
        error!("{}", e);
    }
    init_logging(opts.verbosity);
    let opts = Opts::new(opts, config_file.ok());
    match opts.command {
        Command::Record => record(&opts)?,
        Command::Cut => run_gui(&opts),
        Command::MonitorDbus => monitor_dbus(&opts),
    }
    Ok(())
}

fn record(opts: &Opts) -> Result<()> {
    info!("Using service: {}", opts.service);
    let path = SessionPath(get_new_name(&opts.output_dir));
    let recorder = Recorder::new(&opts, &path)?;
    let _session = recorder.record_new_session()?;
    Ok(())
}

fn monitor_dbus(opts: &Opts) {
    let conn = DbusConnection::new(&opts.service);
    loop {
        for ev in conn.get_new_events() {
            println!("{:?}", ev);
        }
    }
}

fn init_logging(verbosity: usize) {
    let level = match verbosity {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
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

fn run_gui(opts: &Opts) {
    let app = StriputaryGui::new(opts);
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("striputary", native_options, Box::new(|_| Box::new(app)));
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::config_file::ConfigFile;
    use crate::service::Service;
    use crate::CliOpts;
    use crate::Command;
    use crate::Opts;

    fn test_opts() -> CliOpts {
        CliOpts {
            output_dir: Some("".into()),
            service: None,
            sound_server: None,
            verbosity: 0,
            command: Command::Record,
        }
    }

    fn test_config_file() -> ConfigFile {
        ConfigFile {
            output_dir: "from_config_file".into(),
            service: None,
            sound_server: None,
        }
    }

    #[test]
    fn service_set_properly() {
        use Service::*;
        let mut p_opts = CliOpts {
            service: Some(SpotifyChromium),
            ..test_opts()
        };
        let config_file = ConfigFile {
            service: Some(SpotifyChromium),
            ..test_config_file()
        };

        let opts = Opts::new(p_opts.clone(), None);
        assert_eq!(opts.service, SpotifyChromium);

        p_opts.service = Some(SpotifyNative);
        let opts = Opts::new(p_opts.clone(), None);
        assert_eq!(opts.service, SpotifyNative);

        p_opts.service = None;
        let opts = Opts::new(p_opts.clone(), None);
        assert_eq!(opts.service, Service::default());

        p_opts.service = None;
        let opts = Opts::new(p_opts.clone(), Some(config_file));
        assert_eq!(opts.service, SpotifyChromium);
    }

    #[test]
    fn output_dir_set_properly() {
        let mut p_opts = CliOpts {
            output_dir: Some("from_cli".into()),
            ..test_opts()
        };
        let config_file = ConfigFile {
            output_dir: "from_config_file".into(),
            ..test_config_file()
        };
        let opts = Opts::new(p_opts.clone(), None);
        assert!(opts.output_dir == Path::new("from_cli").to_owned());
        let opts = Opts::new(p_opts.clone(), Some(config_file.clone()));
        assert!(opts.output_dir == Path::new("from_cli").to_owned());
        p_opts.output_dir = None;
        let opts = Opts::new(p_opts.clone(), Some(config_file));
        assert!(opts.output_dir == Path::new("from_config_file").to_owned());
    }

    #[test]
    #[should_panic(expected = "Need an output folder")]
    fn panic_if_output_dir_not_set() {
        let p_opts = CliOpts {
            output_dir: None,
            service: None,
            sound_server: None,
            verbosity: 0,
            command: Command::Record,
        };
        let _opts = Opts::new(p_opts.clone(), None);
    }
}
