use std::fmt::Display;
use std::str::FromStr;

use serde::Deserialize;
use serde::Serialize;

#[derive(Copy, Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub enum OutputFormat {
    AAC,
    Mp3,
    #[default]
    Opus,
}

impl FromStr for OutputFormat {
    type Err = serde_yaml::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // This is quite ugly, but ensures that the config file string representation
        // is the same as in the command line options (which uses the FromStr impl),
        // without adding any additional dependencies
        serde_yaml::from_str(s)
    }
}

impl Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Similar to the from_str implementation, this is ugly but consistent.
        let s = serde_yaml::to_string(self).unwrap();
        let s = s.trim();
        write!(f, "{}", s)
    }
}

impl OutputFormat {
    pub fn ffmpeg_libname(&self) -> &str {
        match self {
            Self::AAC => "aac",
            Self::Mp3 => "libmp3lame",
            Self::Opus => "libopus",
        }
    }

    pub fn format_ending(&self) -> &str {
        match self {
            Self::AAC => "aac",
            Self::Mp3 => "mp3",
            Self::Opus => "opus",
        }
    }
}
