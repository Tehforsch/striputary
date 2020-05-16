pub static SINK_NAME: &'static str = "spotifyrec";
pub static SINK_SOURCE_NAME: &'static str = "Spotify";
pub static DEFAULT_BUFFER_FILE: &'static str = "buffer.wav";
pub static DEFAULT_SESSION_FILE: &'static str = "session.yaml";
pub static TIME_BEFORE_SESSION_START: f64 = 10.0;

pub static BITRATE: i64 = 320;
pub static MIN_OFFSET: f64 = -2.;
pub static MAX_OFFSET: f64 = 0.;
pub static READ_BUFFER: f64 = 0.5;
pub static NUM_OFFSETS_TO_TRY: i64 = 1000;
pub static NUM_SAMPLES_PER_AVERAGE_VOLUME: usize = 300;
