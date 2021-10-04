use crate::{run_args::RunArgs, song::Song};

use super::recording_thread_handle::RecordingThreadHandle;

pub enum RecordingThreadHandleStatus {
    Running(RecordingThreadHandle),
    Failed(anyhow::Error),
    Stopped,
}

impl RecordingThreadHandleStatus {
    pub fn update(&mut self) {
        take_mut::take(self, |tmp_self| {
            let thread_failed = match tmp_self {
                Self::Running(ref thread) => !thread.check_still_running(),
                _ => false,
            };

            if thread_failed {
                if let Self::Running(thread) = tmp_self {
                    let result = thread.get_result();
                    return match result {
                        Ok(_) => Self::Stopped,
                        Err(error) => Self::Failed(error),
                    };
                } else {
                    panic!("Impossible");
                }
            }
            tmp_self
        });
    }
}

pub struct FallibleRecordingThreadHandle {
    pub status: RecordingThreadHandleStatus,
    recorded_songs: Vec<Song>,
}

impl FallibleRecordingThreadHandle {
    pub fn new_stopped() -> Self {
        Self {
            status: RecordingThreadHandleStatus::Stopped,
            recorded_songs: vec![],
        }
    }

    pub fn new_running(run_args: &RunArgs) -> Self {
        Self {
            status: RecordingThreadHandleStatus::Running(RecordingThreadHandle::new(run_args)),
            recorded_songs: vec![],
        }
    }

    pub fn update(&mut self) {
        self.status.update();
        self.update_songs();
    }

    fn update_songs(&mut self) {
        if let RecordingThreadHandleStatus::Running(ref thread) = self.status {
            if let Some(song) = thread.get_new_songs() {
                self.recorded_songs.push(song)
            }
        }
    }

    pub fn is_running(&self) -> bool {
        match self.status {
            RecordingThreadHandleStatus::Running(_) => true,
            _ => false,
        }
    }
}
