pub mod args;
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
pub mod run_args;
pub mod service_config;
mod sink_type;
pub mod song;
pub mod wav;

use std::path::Path;

use anyhow::Result;
use args::Opts;
use clap::Parser;
use config_file::ConfigFile;
use service_config::Service;
use sink_type::SinkType;

use crate::gui::StriputaryGui;

fn main() -> Result<(), anyhow::Error> {
    let args = Opts::parse();
    let config_file = ConfigFile::read();
    if let Err(ref e) = config_file {
        println!("{}", e);
    }
    let config_file = config_file.ok();
    let output_dir = args
        .output_dir
        .or(config_file.as_ref().map(|file| file.output_dir.clone()));
    let service = args
        .service
        .or(config_file
            .as_ref()
            .and_then(|file: &ConfigFile| file.service))
        .unwrap_or_else(|| {
            let service = Service::default();
            println!("No service specified in command line args or config file. Using default.");
            service
        });
    let monitor = args.monitor
        | config_file
            .as_ref()
            .and_then(|file: &ConfigFile| file.monitor)
            .unwrap_or(false);
    let sink_type = if monitor {
        SinkType::Monitor
    } else {
        SinkType::Normal
    };
    println!("Using service: {}", service);
    match output_dir {
        Some(dir) => {
            Ok(run_gui(&dir, service, sink_type))
        }
        None => panic!("Need an output folder - either pass it as a command line argument or specify it in the config file (probably ~/.config/striputary/config.yaml")
    }
}

fn run_gui(dir: &Path, service: Service, sink_type: SinkType) {
    let app = StriputaryGui::new(dir, service, sink_type);
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("striputary", native_options, Box::new(|_| Box::new(app)));
}
