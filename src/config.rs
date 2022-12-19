use std::time::Duration;

pub static CONFIG_FILE_NAME: &str = "config.yaml";

pub static STRIPUTARY_SINK_NAME: &str = "striputary";

pub static DEFAULT_BUFFER_FILE: &str = "buffer.wav";
pub static DEFAULT_SESSION_FILE: &str = "session.yaml";
pub static DEFAULT_MUSIC_DIR: &str = "music";

pub static DEFAULT_SERVICE: &str = "spotify";
// This should be more than 3-4 seconds at least
pub static TIME_BEFORE_SESSION_START: Duration = Duration::from_secs(5);
pub static WAIT_TIME_BEFORE_FIRST_SONG: Duration = Duration::from_secs(1);
pub static TIME_AFTER_SESSION_END: Duration = Duration::from_secs(10);

pub static TIME_WITHOUT_DBUS_SIGNAL_BEFORE_STOPPING: Duration = Duration::from_secs(10);
pub static TIME_BETWEEN_SUBSEQUENT_DBUS_COMMANDS: Duration = Duration::from_secs(1);

pub static BITRATE: i64 = 192000;
pub static MIN_OFFSET: f64 = -3.;
pub static MAX_OFFSET: f64 = 3.;
pub static READ_BUFFER: f64 = 0.5;
pub static NUM_OFFSETS_TO_TRY: i64 = 1000;
pub static NUM_SAMPLES_PER_AVERAGE_VOLUME: usize = 2000;

pub static NUM_PLOT_DATA_POINTS: i64 = 500;

pub static RECV_CUT_SONG_TIMEOUT: Duration = Duration::from_millis(2);
pub static RECV_CUT_INFO_TIMEOUT: Duration = Duration::from_millis(2);
pub static RECV_RECORDED_SONG_TIMEOUT: Duration = Duration::from_millis(2);
pub static RECV_RECORDED_SESSION_TIMEOUT: Duration = Duration::from_millis(2);
