use crate::config::{
    TIME_AFTER_SESSION_END, TIME_BEFORE_SESSION_START, WAIT_TIME_BEFORE_FIRST_SONG,
};
use crate::dbus::{collect_dbus_info, previous_song, start_playback, stop_playback};
use crate::recording_session::RecordingSession;
use crate::run_args::RunArgs;
use crate::{recorder, yaml_session};
use anyhow::{anyhow, Context, Result};
use std::fs::create_dir_all;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

#[derive(PartialEq)]
pub enum RecordingExitStatus {
    FinishedOrInterrupted,
    AlbumFinished,
}

#[derive(PartialEq)]
pub enum RecordingStatus {
    Running,
    Finished(RecordingExitStatus),
}

pub struct RecordingThreadHandle {
    handle: JoinHandle<Result<()>>,
    is_running: Arc<AtomicBool>,
}

impl RecordingThreadHandle {
    pub fn new(run_args: &RunArgs) -> Self {
        let is_running = Arc::new(AtomicBool::new(true));
        let mut thread = RecordingThread::new(is_running.clone(), run_args);
        let handle = thread::spawn(move || thread.record_sessions_and_save_session_files());
        Self { handle, is_running }
    }

    pub fn check_still_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    pub fn get_result(self) -> Result<()> {
        self.handle.join().unwrap()
    }
}

pub enum FallibleRecordingThreadHandle {
    Running(RecordingThreadHandle),
    Failed(anyhow::Error),
    Stopped,
}

impl FallibleRecordingThreadHandle {
    pub fn new(run_args: &RunArgs) -> Self {
        Self::Running(RecordingThreadHandle::new(run_args))
    }

    pub fn handle(&mut self) {
        take_mut::take(self, |tmp_self| {
            let thread_failed = match tmp_self {
                FallibleRecordingThreadHandle::Running(ref thread) => !thread.check_still_running(),
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

    pub fn is_running(&self) -> bool {
        match self {
            Self::Running(_) => true,
            _ => false,
        }
    }
}

pub struct RecordingThread {
    run_args: RunArgs,
    recorded_sessions: Vec<RecordingSession>,
    is_running: Arc<AtomicBool>,
}

impl RecordingThread {
    pub fn new(is_running: Arc<AtomicBool>, run_args: &RunArgs) -> Self {
        Self {
            run_args: run_args.clone(),
            recorded_sessions: vec![],
            is_running,
        }
    }

    pub fn record_sessions_and_save_session_files(&mut self) -> Result<()> {
        let result = (|| {
            for num in 0.. {
                let session_file = self.run_args.get_yaml_file(num);
                let buffer_file = self.run_args.get_buffer_file(num);
                let (status, session) = self.record_new_session(&session_file, &buffer_file)?;
                yaml_session::save(&session)?;
                if status == RecordingExitStatus::FinishedOrInterrupted {
                    break;
                }
            }
            Ok(())
        })();
        self.is_running.store(false, Ordering::SeqCst);
        result
    }

    pub fn record_new_session(
        &self,
        session_file: &Path,
        buffer_file: &Path,
    ) -> Result<(RecordingExitStatus, RecordingSession)> {
        create_dir_all(&self.run_args.session_dir).context("Failed to create session directory")?;
        if buffer_file.exists() {
            return Err(anyhow!(
                "Buffer file already exists, not recording a new session."
            ));
        }
        let recording_handles =
            recorder::start_recording(&buffer_file, &self.run_args.service_config)?;
        let record_start_time = Instant::now();
        // Check for dbus signals while recording until terminated
        let (status, session) = self.polling_loop(session_file, &record_start_time)?;
        // Whether the loop ended because of the user interrupt (ctrl-c) or
        // because the playback was stopped doesn't matter - kill the recording processes
        recorder::stop_recording(recording_handles)?;
        Ok((status, session))
    }

    fn polling_loop(
        &self,
        session_file: &Path,
        record_start_time: &Instant,
    ) -> Result<(RecordingExitStatus, RecordingSession)> {
        self.initial_buffer_phase()?;
        let (status, session) = self.recording_phase(session_file, record_start_time)?;
        self.final_buffer_phase();
        Ok((status, session))
    }

    fn initial_buffer_phase(&self) -> Result<()> {
        // Add a small time buffer before starting the playback properly.
        // This ensures that something starts playing, thus registering the
        // pulse audio sink. Also it avoids overflows when calculating the offset
        println!("Begin pre-session phase");
        start_playback(&self.run_args.service_config)?;
        thread::sleep(Duration::from_secs_f64(TIME_BEFORE_SESSION_START));
        stop_playback(&self.run_args.service_config)?;
        println!("Go to beginning of song");
        previous_song(&self.run_args.service_config)?;
        thread::sleep(Duration::from_secs_f64(WAIT_TIME_BEFORE_FIRST_SONG));
        Ok(())
    }

    fn recording_phase(
        &self,
        session_file: &Path,
        record_start_time: &Instant,
    ) -> Result<(RecordingExitStatus, RecordingSession)> {
        let recording_start_time = Instant::now()
            .duration_since(*record_start_time)
            .as_secs_f64();
        let mut session = RecordingSession::new(&session_file, recording_start_time);
        println!("Start playback.");
        start_playback(&self.run_args.service_config)?;
        loop {
            let playback_status = collect_dbus_info(&mut session, &self.run_args.service_config)?;
            if let RecordingStatus::Finished(exit_status) = playback_status {
                return Ok((exit_status, session));
            }
        }
    }

    fn final_buffer_phase(&self) {
        println!("Recording finished. Record final buffer for a few seconds");
        thread::sleep(Duration::from_secs_f64(TIME_AFTER_SESSION_END));
    }
}
