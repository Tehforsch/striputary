use iced::{
    event::Status,
    mouse::{self, Button},
    theme::Palette,
    widget::canvas::{path::Builder, Event, Frame, Geometry, Path, Program, Stroke},
    Color, Point, Rectangle, Renderer, Theme,
};
use ordered_float::OrderedFloat;

use crate::{
    audio::{get_volume_at, AudioTime, CutInfo, WavFileReader},
    song::Song,
};

const NUM_PLOT_POINTS: usize = 100;
const PLOT_STROKE_WIDTH: f32 = 1.5;
const PLOT_COLOR: Color = Palette::GRUVBOX_DARK.primary;
const MARKER_STROKE_WIDTH: f32 = 2.5;
const MARKER_COLOR: Color = Palette::GRUVBOX_DARK.danger;

type Volume = f32;

struct Data {
    time: AudioTime,
    /// The volume of this data point normalized
    /// by the maximum volume of all points in this
    /// plot.
    volume: Volume,
}

struct Bounds {
    min_time: AudioTime,
    max_time: AudioTime,
    height: f32,
    width: f32,
}

pub fn interpolate(start: AudioTime, end: AudioTime, factor: f32) -> AudioTime {
    start + (end - start) * factor as f64
}

pub fn interpolation_factor(start: AudioTime, end: AudioTime, x: AudioTime) -> f64 {
    (x.time - start.time) / (end.time - start.time)
}

impl Bounds {
    fn x_pos_to_time(&self, pos: Point) -> AudioTime {
        let f = pos.x / self.width;
        interpolate(self.min_time, self.max_time, f)
    }

    fn data_to_point(&self, data: &Data) -> Point {
        let f = interpolation_factor(self.min_time, self.max_time, data.time);
        Point::new(f as f32 * self.width, data.volume * self.height)
    }
}

impl Data {}

pub struct Plot {
    volume_data: Vec<Data>,
    song_before: Option<Song>,
    song_after: Option<Song>,
    cut_time: AudioTime,
    finished_cut_before: bool,
    finished_cut_after: bool,
}

impl Plot {
    pub fn new(
        reader: &mut WavFileReader,
        song_before: Option<Song>,
        song_after: Option<Song>,
        cut_time: AudioTime,
    ) -> Self {
        let delta = AudioTime::from_time_same_spec(3.0, cut_time);
        let start = cut_time - delta;
        let end = cut_time + delta;
        let volume_data: Vec<_> = (0..NUM_PLOT_POINTS)
            .map(|i| {
                let f = i as f32 / NUM_PLOT_POINTS as f32;
                let time = interpolate(start, end, f);
                let volume = get_volume_at(reader, time).unwrap_or(0.0) as f32;
                Data { time, volume }
            })
            .collect();
        // Normalize
        let max_volume = volume_data
            .iter()
            .map(|data| OrderedFloat(data.volume))
            .max()
            .unwrap();
        let volume_data = volume_data
            .into_iter()
            .map(|data| Data {
                time: data.time,
                volume: data.volume / *max_volume,
            })
            .collect();
        Self {
            volume_data,
            song_before,
            song_after,
            cut_time,
            finished_cut_before: false,
            finished_cut_after: false,
        }
    }

    fn get_plot_path(&self, data: &[Data], bounds: &Bounds) -> Path {
        let mut path = Builder::new();

        if data.len() > 0 {
            path.move_to(bounds.data_to_point(&data[0]));
            for data in data.iter() {
                path.line_to(bounds.data_to_point(data));
            }
        }
        path.build()
    }

    /// Return the path left of the marker and the path right
    /// of the marker, so they can be colored individually.
    fn get_plot_paths(&self, bounds: &Bounds) -> (Path, Path) {
        let cutoff = self
            .volume_data
            .iter()
            .enumerate()
            .find(|(_, data)| data.time > self.cut_time)
            .map(|(i, _)| i)
            .unwrap_or(self.volume_data.len() - 1);
        (
            self.get_plot_path(&self.volume_data[..=cutoff], bounds),
            self.get_plot_path(&self.volume_data[cutoff..], bounds),
        )
    }

    fn get_marker_path(&self, bounds: &Bounds) -> Path {
        let mut path = Builder::new();
        path.move_to(bounds.data_to_point(&Data {
            time: self.cut_time,
            volume: -1.0,
        }));
        path.line_to(bounds.data_to_point(&Data {
            time: self.cut_time,
            volume: 1.0,
        }));
        path.build()
    }

    pub fn set_cut_position(&mut self, time: AudioTime) {
        self.cut_time = time;
        self.finished_cut_before = false;
        self.finished_cut_after = false;
    }

    pub fn update_on_finished_cut(&mut self, cut_song: &CutInfo) {
        if self.song_before.as_ref() == Some(&cut_song.cut.song) {
            self.finished_cut_before = true;
        }
        if self.song_after.as_ref() == Some(&cut_song.cut.song) {
            self.finished_cut_after = true;
        }
    }

    pub fn song_after(&self) -> Option<&Song> {
        self.song_after.as_ref()
    }

    pub fn song_before(&self) -> Option<&Song> {
        self.song_before.as_ref()
    }

    fn get_bounds(&self, bounds: Rectangle) -> Bounds {
        Bounds {
            width: bounds.width,
            height: bounds.height,
            min_time: self.min_time(),
            max_time: self.max_time(),
        }
    }

    pub fn max_time(&self) -> AudioTime {
        self.volume_data.last().unwrap().time
    }

    fn min_time(&self) -> AudioTime {
        self.volume_data.first().unwrap().time
    }

    pub fn cut_time(&self) -> AudioTime {
        self.cut_time
    }
}

pub struct PlotMarkerMoved {
    pub time: AudioTime,
}

impl Program<PlotMarkerMoved> for Plot {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: Event,
        rect: Rectangle,
        cursor: mouse::Cursor,
    ) -> (Status, Option<PlotMarkerMoved>) {
        let bounds = self.get_bounds(rect);
        if let Event::Mouse(mouse::Event::ButtonPressed(ev)) = event {
            if ev == Button::Left {
                if let Some(pos) = cursor.position_in(rect) {
                    return (
                        Status::Ignored,
                        Some(PlotMarkerMoved {
                            time: bounds.x_pos_to_time(pos),
                        }),
                    );
                }
            }
        }
        (Status::Ignored, None)
    }

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        rect: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, rect.size());

        let bounds = self.get_bounds(rect);
        let (plot_before, plot_after) = self.get_plot_paths(&bounds);
        let color = |finished_cutting| {
            if finished_cutting {
                Color::from_rgb(0.0, 0.8, 0.0)
            } else {
                PLOT_COLOR
            }
        };
        frame.stroke(
            &plot_before,
            Stroke::default()
                .with_width(PLOT_STROKE_WIDTH)
                .with_color(color(self.finished_cut_before)),
        );
        frame.stroke(
            &plot_after,
            Stroke::default()
                .with_width(PLOT_STROKE_WIDTH)
                .with_color(color(self.finished_cut_after)),
        );
        let marker = self.get_marker_path(&bounds);
        frame.stroke(
            &marker,
            Stroke::default()
                .with_width(MARKER_STROKE_WIDTH)
                .with_color(MARKER_COLOR),
        );

        vec![frame.into_geometry()]
    }
}
