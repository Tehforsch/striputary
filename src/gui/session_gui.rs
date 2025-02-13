use crate::audio::{
    AudioTime, CutInfo, Cutter, CuttingStrategy, DbusLengthsStrategy, Manual, WavFileReader,
};
use crate::recording_session::{RecordingSession, RecordingSessionWithPath, SessionPath};
use crate::song::Song;
use anyhow::Result;
use futures::StreamExt;
use iced::alignment::Horizontal;
use iced::stream::channel;
use iced::widget::{column, horizontal_space, text, Canvas, Column, Row};
use iced::Length::Fill;
use iced::{Element, Subscription, Task};
use log::debug;

use super::plot::{Plot, PlotMarkerMoved};

const CANVAS_HEIGHT: f32 = 80.0;

#[derive(Clone, Debug)]
pub enum SessionMessage {
    SetCutPosition(SetCutPosition),
    CutSongs,
    FinishedCutting(CutInfo),
}

#[derive(Clone, Debug)]
pub struct SetCutPosition {
    cut_index: usize,
    time: AudioTime,
}

pub struct SessionGui {
    plots: Vec<Plot>,
    reader: WavFileReader,
    session: RecordingSession,
    path: SessionPath,
    cuts: Manual,
    cutting_state: CuttingState,
}

enum CuttingState {
    Waiting,
    Cutting,
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
        Ok(Self {
            reader,
            session: session.session,
            plots,
            cuts,
            path,
            cutting_state: CuttingState::Waiting,
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
                    .push_maybe(plot.song_before().map(make_title))
                    .push(horizontal_space())
                    .push_maybe(plot.song_after().map(make_title));
                column![titles, canvas].into()
            })
            .collect();
        Column::with_children(canvases).into()
    }

    pub fn set_cut_position(&mut self, pos: SetCutPosition) {
        self.cuts.0[pos.cut_index] = pos.time;
        self.plots[pos.cut_index].set_cut_position(pos.time);
    }

    pub fn cut_current_session(&mut self) {
        self.cutting_state = CuttingState::Cutting;
    }

    pub fn subscription(&self) -> Subscription<SessionMessage> {
        let path = self.path.0.clone();
        let cuts = self.cuts.clone();
        match self.cutting_state {
            CuttingState::Cutting => Subscription::run_with_id(
                "cut",
                channel(5, move |sender| Cutter::run(path, cuts, sender))
                    .map(|m| SessionMessage::FinishedCutting(m)),
            ),
            CuttingState::Waiting => Subscription::none(),
        }
    }
}
