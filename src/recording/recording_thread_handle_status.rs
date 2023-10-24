use super::recording_thread_handle::AsyncRecorder;
use crate::recording_session::SessionPath;
use crate::song::Song;
use crate::Opts;

pub enum RecordingThreadHandleStatus {
    Running(AsyncRecorder),
    Failed(anyhow::Error),
    Stopped,
}

impl RecordingThreadHandleStatus {
    pub fn update(&mut self) {
        take_mut::take(self, |tmp_self| {
            let thread_failed = match tmp_self {
                Self::Running(ref thread) => !thread.is_running(),
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
                    unreachable!()
                }
            }
            tmp_self
        });
        if let Self::Running(ref mut thread) = self {
            thread.update();
        }
    }

    pub fn new_stopped() -> Self {
        Self::Stopped
    }

    pub fn new_running(opts: &Opts, path: &SessionPath) -> Self {
        Self::Running(AsyncRecorder::new(opts, path))
    }

    pub fn is_running(&self) -> bool {
        matches!(self, RecordingThreadHandleStatus::Running(_))
    }

    pub fn get_songs(&self) -> &[Song] {
        match self {
            Self::Running(thread) => thread.songs.get_data(),
            _ => &[],
        }
    }
}
