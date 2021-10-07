use super::recording_thread_handle::RecordingThreadHandle;
use crate::run_args::RunArgs;
use crate::song::Song;

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
        if let Self::Running(ref mut thread) = self {
            thread.update();
        }
    }

    pub fn new_stopped() -> Self {
        Self::Stopped
    }

    pub fn new_running(run_args: &RunArgs) -> Self {
        Self::Running(RecordingThreadHandle::new(run_args))
    }

    pub fn is_running(&self) -> bool {
        match self {
            RecordingThreadHandleStatus::Running(_) => true,
            _ => false,
        }
    }

    pub fn get_songs(&self) -> &[Song] {
        match self {
            Self::Running(thread) => thread.songs.get_data(),
            _ => &[],
        }
    }
}
