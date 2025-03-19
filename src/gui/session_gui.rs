use std::sync::Mutex;

use crate::audio::{
    playback::play_audio, playback::Progress, playback::WavSource, AudioTime, CutInfo, Cutter,
    CuttingStrategy, DbusLengthsStrategy, Manual, WavFileReader,
};
use crate::recording_session::{RecordingSession, RecordingSessionWithPath, SessionPath};
use crate::song::Song;
use anyhow::Result;
use futures::StreamExt;
use iced::alignment::Horizontal;
use iced::keyboard::key;
use iced::stream::channel;
use iced::widget::{column, horizontal_space, text, Canvas, Column, Row, Space};
use iced::Length::Fill;
use iced::{keyboard, Element, Subscription};
use log::debug;

use super::plot::{Plot, PlotMarkerMoved};

pub const CANVAS_HEIGHT: f32 = 80.0;
const LEFT_PLOT_TEXT_MARGIN: f32 = 50.0;
const RIGHT_PLOT_TEXT_MARGIN: f32 = 50.0;

#[derive(Clone, Debug)]
pub enum SessionMessage {
    SetCutPosition(SetCutPosition),
    CutSongs,
    FinishedCutting(CutInfo),
    PlaybackSong,
    UpdateProgress(Progress),
}

#[derive(Clone, Debug)]
pub struct SetCutPosition {
    cut_index: usize,
    time: AudioTime,
}

pub struct SessionGui {
    plots: Vec<Plot>,
    reader: Mutex<WavFileReader>,
    path: SessionPath,
    cuts: Manual,
    cutting_state: CuttingState,
    playback_state: PlaybackState,
    last_touched_song: usize,
    playback_index: usize,
}

enum CuttingState {
    Waiting,
    Cutting,
}

enum PlaybackState {
    Waiting,
    Playback(PlaybackInfo),
}

#[derive(Clone, Debug)]
struct PlaybackInfo {
    start: AudioTime,
    end: AudioTime,
    index: usize,
}

fn load_plots(reader: &mut WavFileReader, session: &RecordingSession) -> Vec<Plot> {
    // The initial guess for the timestamps
    let timestamps = (DbusLengthsStrategy).get_timestamps(reader, session);
    let mut plots: Vec<_> = (0..session.songs.len())
        .map(|i| {
            let before = if i == 0 {
                None
            } else {
                Some(&session.songs[i])
            };
            let after = session.songs.get(i + 1);
            let timing = timestamps[i];
            Plot::new(reader, before.cloned(), after.cloned(), timing)
        })
        .collect();
    if !session.songs.is_empty() {
        plots.push(Plot::new(
            reader,
            session.songs.last().cloned(),
            None,
            *timestamps.last().unwrap(),
        ))
    }
    plots
}

impl SessionGui {
    pub fn new(path: SessionPath) -> Result<SessionGui> {
        let session = RecordingSessionWithPath::load_from_dir(&path.0)?;
        let mut reader = hound::WavReader::open(session.path.get_buffer_file())?;
        let plots = load_plots(&mut reader, &session.session);
        let cuts = Manual::new(&mut reader, &session.session, DbusLengthsStrategy);
        let playback_state = PlaybackState::Waiting;
        Ok(Self {
            reader: Mutex::new(reader),
            plots,
            cuts,
            path,
            cutting_state: CuttingState::Waiting,
            last_touched_song: 0,
            playback_state,
            playback_index: 0,
        })
    }

    pub fn update(&mut self, m: SessionMessage) {
        match m {
            SessionMessage::SetCutPosition(pos) => {
                self.set_cut_position(pos);
            }
            SessionMessage::CutSongs => {
                return self.cut_current_session();
            }
            SessionMessage::FinishedCutting(cut) => {
                for plot in self.plots.iter_mut() {
                    plot.update_on_finished_cut(&cut);
                }
                debug!("Finished {}", cut.cut.song);
            }
            SessionMessage::PlaybackSong => self.playback_song(),
            SessionMessage::UpdateProgress(_) => todo!(),
        }
    }

    pub fn view(&self) -> Element<SessionMessage> {
        let canvases: Vec<_> = self
            .plots
            .iter()
            .enumerate()
            .map(|(i, plot)| {
                let c: Element<PlotMarkerMoved> =
                    Canvas::new(plot).width(Fill).height(CANVAS_HEIGHT).into();
                let canvas = c.map(move |message| {
                    SessionMessage::SetCutPosition(SetCutPosition {
                        cut_index: i,
                        time: message.time,
                    })
                });
                let make_title = |song: &Song| {
                    text(song.to_string_short().to_string()).align_x(Horizontal::Left)
                };
                let titles = Row::new()
                    .push(Space::with_width(LEFT_PLOT_TEXT_MARGIN))
                    .push_maybe(plot.song_before().map(make_title))
                    .push(horizontal_space())
                    .push_maybe(plot.song_after().map(make_title))
                    .push(Space::with_width(RIGHT_PLOT_TEXT_MARGIN));
                column![titles, canvas].into()
            })
            .collect();
        Column::with_children(canvases).into()
    }

    fn playback_song(&mut self) {
        let info = PlaybackInfo {
            start: self.plots[self.last_touched_song].cut_time(),
            end: self.plots[self.last_touched_song].max_time(),
            index: self.next_playback_index(),
        };
        self.playback_state = PlaybackState::Playback(info);
    }

    pub fn set_cut_position(&mut self, pos: SetCutPosition) {
        self.cuts.0[pos.cut_index] = pos.time;
        self.plots[pos.cut_index].set_cut_position(pos.time);
        self.last_touched_song = pos.cut_index;
    }

    pub fn cut_current_session(&mut self) {
        self.cutting_state = CuttingState::Cutting;
    }

    fn keyboard_listener(&self) -> Subscription<SessionMessage> {
        keyboard::on_key_press(|key, _| {
            let keyboard::Key::Named(key) = key else {
                return None;
            };

            match key {
                key::Named::Space => Some(SessionMessage::PlaybackSong),
                _ => None,
            }
        })
    }

    pub fn subscription(&self) -> Subscription<SessionMessage> {
        let path = self.path.0.clone();
        let cuts = self.cuts.clone();
        let mut subscriptions = vec![];
        if let CuttingState::Cutting = self.cutting_state {
            subscriptions.push(Subscription::run_with_id(
                "cut",
                channel(5, move |sender| Cutter::run(path, cuts, sender))
                    .map(|m| SessionMessage::FinishedCutting(m)),
            ))
        }
        if let PlaybackState::Playback(ref info) = self.playback_state {
            let source = self.get_wav_source(info);
            let index = info.index;
            subscriptions.push(Subscription::run_with_id(
                format!("playback_{index}"),
                channel(5, move |sender| play_audio(source, sender))
                    .map(|m| SessionMessage::UpdateProgress(m)),
            ))
        }
        subscriptions.push(self.keyboard_listener());
        Subscription::batch(subscriptions)
    }

    fn next_playback_index(&mut self) -> usize {
        self.playback_index += 1;
        self.playback_index
    }

    fn get_wav_source(&self, info: &PlaybackInfo) -> WavSource {
        WavSource::new(&mut self.reader.lock().unwrap(), info.start, info.end)
    }
}
