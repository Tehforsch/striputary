pub static SINK_NAME: &str = "spotifyrec";
pub static SINK_SOURCE_NAME: &str = "Spotify";
pub static DEFAULT_BUFFER_FILE: &str = "buffer.wav";
pub static DEFAULT_SESSION_FILE: &str = "session.yaml";
// This should be more than 3-4 seconds at least
pub static TIME_BEFORE_SESSION_START: f64 = 5.0;
pub static WAIT_TIME_BEFORE_FIRST_SONG: f64 = 1.0;
pub static TIME_AFTER_SESSION_END: f64 = 10.0;

pub static BITRATE: i64 = 320;
pub static MIN_OFFSET: f64 = -2.;
pub static MAX_OFFSET: f64 = 0.;
pub static READ_BUFFER: f64 = 0.5;
pub static NUM_OFFSETS_TO_TRY: i64 = 1000;
pub static NUM_SAMPLES_PER_AVERAGE_VOLUME: usize = 300;
