use crate::recording_session::RecordingSession;
use crate::song::Song;
use std::path::PathBuf;

pub fn cut_session(session: RecordingSession) {
    let start_iter = session.timestamps.iter();
    let mut end_iter = session.timestamps.iter();
    end_iter.next().unwrap();
    dbg!(&session.timestamps);
    for ((start_time, end_time), song) in start_iter.zip(end_iter).zip(session.songs.iter()) {
        dbg!(song, start_time, end_time);
        cut_song(session.get_buffer_file(), song, 0.0, 0.0);
    }
}

pub fn cut_song(source_file: PathBuf, song: Song, start_time: f64, end_time: f64) {
    dbg!(source_file, song, start_time, end_time);
}
