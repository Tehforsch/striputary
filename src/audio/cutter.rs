use std::{
    path::{Path, PathBuf},
    process::Child,
};

use futures::{channel::mpsc::Sender, SinkExt};
use log::{error, warn};

use crate::recording_session::{RecordingSession, RecordingSessionWithPath};

use super::{
    cut::{get_cut_command, CutInfo},
    cutting_strategy::CuttingStrategy,
    sample_reader::{get_volume_at, WavFileReader},
    time::AudioTime,
};

const MAX_NUM_PROCESSES: usize = 10;

fn get_cuts(path: &Path, strategy: impl CuttingStrategy) -> Vec<CutInfo> {
    let mut session = RecordingSessionWithPath::load_from_dir(path).unwrap();
    let mut reader = hound::WavReader::open(session.path.get_buffer_file()).unwrap();
    filter_invalid_songs(&mut reader, &mut session.session);
    let timestamps = strategy.get_timestamps(&mut reader, &session.session);
    assert_eq!(timestamps.len(), session.session.songs.len() + 1);
    let cuts = timestamps
        .iter()
        .zip(timestamps[1..].iter())
        .zip(session.session.songs.iter())
        .map(|((start, end), song)| CutInfo::new(&session, song, *start, *end))
        .collect();
    cuts
}

fn filter_invalid_songs(reader: &mut WavFileReader, session: &mut RecordingSession) {
    let last_valid_timestamp = session
        .timestamps
        .iter()
        .enumerate()
        .take_while(|(_, timestamp)| {
            let time = AudioTime::from_time_and_spec(timestamp.in_secs(), reader.spec());
            get_volume_at(reader, time).is_ok()
        })
        .map(|(index, _)| index)
        .last();
    match last_valid_timestamp {
        None => error!("No valid timestamp. Most likely a faulty recording"),
        Some(0) => error!("Only one valid timestamp. Most likely a faulty recording"),
        Some(index) => {
            let drained_songs = session.songs.drain(index - 1..);
            session.timestamps.drain(index..);
            for song in drained_songs {
                warn!("Found invalid song: {song:?}");
            }
        }
    }
}

pub struct Cutter {
    cuts: Vec<CutInfo>,
    handles: Vec<(Child, CutInfo)>,
    sender: Sender<CutInfo>,
}

impl Cutter {
    pub async fn run(path: PathBuf, strategy: impl CuttingStrategy, sender: Sender<CutInfo>) {
        let cuts = get_cuts(&path, strategy);
        Self {
            cuts,
            handles: vec![],
            sender,
        }
        .run_internal()
        .await
    }

    fn pop_front(&mut self) -> Option<CutInfo> {
        if self.cuts.is_empty() {
            None
        } else {
            Some(self.cuts.remove(0))
        }
    }

    pub async fn run_internal(mut self) {
        // TODO await here
        while !self.handles.is_empty() || !self.cuts.is_empty() {
            if self.handles.len() < MAX_NUM_PROCESSES {
                if let Some(cut) = self.pop_front() {
                    let mut command = get_cut_command(&cut).unwrap();
                    self.handles
                        .push((command.spawn().expect("Failed to cut song."), cut.clone()));
                }
            }
            let mut finished: Vec<_> = self
                .handles
                .iter_mut()
                .map(|(handle, _)| match handle.try_wait().unwrap() {
                    Some(status) => {
                        if status.success() {
                            true
                        } else {
                            panic!("Cut process failed with {}", status);
                        }
                    }
                    None => false,
                })
                .collect();
            let (finished, still_running): (Vec<_>, Vec<_>) =
                self.handles.drain(..).partition(|_| finished.remove(0));

            for (_, cut) in finished {
                self.sender.send(cut).await.unwrap()
            }
            self.handles = still_running;
        }
    }
}
