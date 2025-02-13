use iced::{
    event::Status,
    mouse::{self, Button},
    theme::Palette,
    widget::canvas::{path::Builder, Event, Frame, Geometry, Path, Program, Stroke},
    Color, Point, Rectangle, Renderer, Theme, Vector,
};

use crate::{
    audio::{get_volume_at, interpolate, interpolation_factor, AudioTime, CutInfo, WavFileReader},
    song::Song,
};

const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 50.0;
const NUM_PLOT_POINTS: usize = 100;
const PLOT_STROKE_WIDTH: f32 = 1.5;
const PLOT_COLOR: Color = Palette::GRUVBOX_DARK.text;
const MARKER_STROKE_WIDTH: f32 = 2.5;
const MARKER_COLOR: Color = Palette::GRUVBOX_DARK.primary;

pub struct Plot {
    data: Vec<Point>,
    song_before: Option<Song>,
    song_after: Option<Song>,
    start: AudioTime,
    end: AudioTime,
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
        let data = (0..NUM_PLOT_POINTS)
            .map(|i| {
                let f = i as f32 / NUM_PLOT_POINTS as f32;
                let time = interpolate(start, end, f as f64);
                let vol = get_volume_at(reader, time).unwrap_or(0.0) as f32;
                Point::new(f * WIDTH, HEIGHT / 2.0 + vol * HEIGHT)
            })
            .collect();
        Self {
            data,
            song_before,
            song_after,
            start,
            end,
            cut_time,
            finished_cut_before: false,
            finished_cut_after: false,
        }
    }

    fn pos_to_time(&self, pos: Point) -> AudioTime {
        interpolate(self.start, self.end, (pos.x / WIDTH) as f64)
    }

    fn time_to_pos(&self, time: AudioTime) -> f32 {
        WIDTH * interpolation_factor(self.start, self.end, time) as f32
    }

    pub fn get_plot_path(&self, data: &[Point]) -> Path {
        let mut path = Builder::new();

        if data.len() > 0 {
            path.move_to(data[0]);
            for point in data.iter() {
                path.line_to(*point);
            }
        }
        path.build()
    }

    /// Return the path left of the marker and the path right
    /// of the marker, so they can be colored individually.
    pub fn get_plot_paths(&self) -> (Path, Path) {
        let cutoff = self
            .data
            .iter()
            .enumerate()
            .find(|(_, p)| self.pos_to_time(**p) > self.cut_time)
            .map(|(i, _)| i)
            .unwrap_or(self.data.len() - 1);
        (
            self.get_plot_path(&self.data[..=cutoff]),
            self.get_plot_path(&self.data[cutoff..]),
        )
    }

    pub fn get_marker_path(&self) -> Path {
        let mut path = Builder::new();
        let x = self.time_to_pos(self.cut_time);
        path.move_to(Point::new(x, 0.0));
        path.line_to(Point::new(x, HEIGHT));
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
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (Status, Option<PlotMarkerMoved>) {
        if let Event::Mouse(mouse::Event::ButtonPressed(ev)) = event {
            if ev == Button::Left {
                if let Some(pos) = cursor.position_in(bounds) {
                    return (
                        Status::Ignored,
                        Some(PlotMarkerMoved {
                            time: self.pos_to_time(pos),
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
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        let (plot_before, plot_after) = self.get_plot_paths();
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
        let marker = self.get_marker_path();
        frame.stroke(
            &marker,
            Stroke::default()
                .with_width(MARKER_STROKE_WIDTH)
                .with_color(MARKER_COLOR),
        );

        vec![frame.into_geometry()]
    }
}
