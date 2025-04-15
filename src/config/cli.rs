use std::path::PathBuf;

use clap::{Parser, Subcommand};

use super::{output_format::OutputFormat, Service, SoundServer};

#[derive(Parser, Debug, Clone)]
#[command(version, about)]
#[command(propagate_version = true)]
pub struct CliOpts {
    /// The output directory to record to.
    /// Passing this argument will override the setting
    /// in the config file.
    #[arg(long)]
    pub output_dir: Option<PathBuf>,
    /// The service to record.  Passing this argument will override
    /// the setting in the config file.
    #[arg(long)]
    pub service: Option<Service>,
    /// The sound server to use.  Passing this argument will override
    /// the setting in the config file.
    #[arg(long)]
    pub sound_server: Option<SoundServer>,
    /// The output format of cut songs (AAC, mp3, opus(default)). Passing this argument will override
    /// the setting in the config file.
    #[arg(long)]
    pub output_format: Option<OutputFormat>,
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbosity: u8,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Begin a new recording.
    Record,
    /// Cut a previous recording into individual songs.
    Cut(CutArgs),
    /// Run the interactive gui for precise cutting.
    Gui,
    /// Monitor the incoming d-bus messages. Used for
    /// debugging purposes.
    MonitorDbus,
}

#[derive(Parser, Debug, Clone)]
pub struct CutArgs {
    /// Path of the recording session to cut.
    pub path: PathBuf,
}
