pub mod audio_excerpt;
pub mod audio_time;
pub mod config;
pub mod config_file;
pub mod cut;
pub mod data_stream;
pub mod errors;
pub mod excerpt_collection;
pub mod gui;
pub mod recording;
pub mod recording_session;
pub mod service_config;
pub mod song;
pub mod wav;

use std::path::PathBuf;

use clap::Parser;
use config_file::ConfigFile;
use log::error;
use log::info;
use log::LevelFilter;
use service_config::Service;
use simplelog::ColorChoice;
use simplelog::ConfigBuilder;
use simplelog::LevelPadding;
use simplelog::TermLogger;
use simplelog::TerminalMode;
use time::UtcOffset;

use crate::gui::StriputaryGui;

#[derive(clap::StructOpt, Clone)]
#[clap(version)]
struct ParseOpts {
    pub output_dir: Option<PathBuf>,
    service: Option<Service>,
    pub session_dir: PathBuf,
    #[clap(short, parse(from_occurrences))]
    pub verbosity: usize,
}

#[derive(Clone)]
pub struct Opts {
    pub output_dir: PathBuf,
    service: Service,
    pub session_dir: PathBuf,
}

impl Opts {
    pub fn get_yaml_file(&self) -> PathBuf {
        self.session_dir.join(config::DEFAULT_SESSION_FILE)
    }

    pub fn get_buffer_file(&self) -> PathBuf {
        self.session_dir.join(config::DEFAULT_BUFFER_FILE)
    }

    fn new(opts: ParseOpts, config_file: Option<ConfigFile>) -> Opts {
        let service = opts
            .service
            .or(config_file.as_ref().and_then(|file| file.service))
            .unwrap_or_else(|| {
                let service = Service::default();
                info!(
                    "No service specified in command line options or config file. Using default."
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
            session_dir: opts.session_dir,
        }
    }
}

fn main() {
    let opts = ParseOpts::parse();
    let config_file = ConfigFile::read();
    if let Err(ref e) = config_file {
        error!("{}", e);
    }
    init_logging(opts.verbosity);
    let opts = Opts::new(opts, config_file.ok());
    info!("Using service: {}", opts.service);
    run_gui(&opts);
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
    use crate::service_config::Service;
    use crate::Opts;
    use crate::ParseOpts;

    fn test_opts() -> ParseOpts {
        ParseOpts {
            output_dir: Some("".into()),
            service: None,
            session_dir: "".into(),
            verbosity: 0,
        }
    }

    fn test_config_file() -> ConfigFile {
        ConfigFile {
            output_dir: "from_config_file".into(),
            service: None,
        }
    }

    #[test]
    fn service_set_properly() {
        use Service::*;
        let mut p_opts = ParseOpts {
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
        let mut p_opts = ParseOpts {
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
        let p_opts = ParseOpts {
            output_dir: None,
            service: None,
            session_dir: "".into(),
            verbosity: 0,
        };
        let _opts = Opts::new(p_opts.clone(), None);
    }
}
