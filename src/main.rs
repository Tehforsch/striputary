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
use simplelog::TermLogger;
use simplelog::TerminalMode;

use crate::gui::StriputaryGui;

#[derive(clap::StructOpt)]
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
    TermLogger::init(
        level,
        ConfigBuilder::default().build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();
}

fn run_gui(opts: &Opts) {
    let app = StriputaryGui::new(opts);
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("striputary", native_options, Box::new(|_| Box::new(app)));
}
