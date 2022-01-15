use eframe::egui::plot::*;
use eframe::egui::*;

use super::config;
use crate::audio_time::AudioTime;
use crate::excerpt_collection::NamedExcerpt;
use crate::song::Song;

pub struct ExcerptPlot {
    pub excerpt: NamedExcerpt,
    pub cut_time: AudioTime,
    pub finished_cutting_song_before: bool,
    pub finished_cutting_song_after: bool,
    pub playback_marker: Option<AudioTime>,
}

impl ExcerptPlot {
    pub fn new(excerpt: NamedExcerpt, cut_time: AudioTime) -> Self {
        Self {
            excerpt,
            cut_time,
            finished_cutting_song_before: false,
            finished_cutting_song_after: false,
            playback_marker: None,
        }
    }

    fn get_lines(&self) -> (Line, Line) {
        let x_values = self.excerpt.excerpt.get_sample_times();
        let y_values = self.excerpt.excerpt.get_volume_plot_data();
        let values_iter = x_values
            .into_iter()
            .zip(y_values)
            .map(|(x, y)| Value::new(x, y));
        let (values_before_cut, values_after_cut): (Vec<_>, Vec<_>) =
            values_iter.partition(|value| value.x < self.cut_time.time);
        (
            Line::new(Values::from_values(values_before_cut)),
            Line::new(Values::from_values(values_after_cut)),
        )
    }

    pub fn get_line_color(&self, finished_cutting: bool) -> Color32 {
        match finished_cutting {
            true => config::CUT_LINE_COLOR,
            false => config::UNCUT_LINE_COLOR,
        }
    }

    pub fn show_playback_marker_at(&mut self, audio_time: AudioTime) {
        self.playback_marker = Some(audio_time);
    }

    pub fn hide_playback_marker(&mut self) {
        self.playback_marker = None;
    }

    pub fn mark_cut(&mut self, song: &Song) {
        if let Some(ref song_before) = self.excerpt.song_before {
            if song_before == song {
                self.finished_cutting_song_before = true;
            }
        }
        if let Some(ref song_after) = self.excerpt.song_after {
            if song_after == song {
                self.finished_cutting_song_after = true;
            }
        }
    }

    pub fn move_marker_to_offset(&mut self, offset: AudioTime) {
        self.cut_time = self.excerpt.excerpt.start + offset;
    }

    pub fn show_and_get_offset(
        &self,
        num: usize,
        ui: &mut Ui,
        click_pos: Option<Pos2>,
    ) -> Option<AudioTime> {
        let (line_before, line_after) = self.get_lines();
        let mut plot_pos: Option<_> = None;
        let response = Plot::new(num)
            .legend(Legend::default())
            .view_aspect(config::PLOT_ASPECT)
            .show_x(false)
            .show_y(false)
            .allow_drag(false)
            .allow_zoom(false)
            .show_background(false)
            .show(ui, |plot_ui| {
                plot_ui.line(
                    line_before.color(self.get_line_color(self.finished_cutting_song_before)),
                );
                plot_ui
                    .line(line_after.color(self.get_line_color(self.finished_cutting_song_after)));
                plot_ui.vline(VLine::new(self.cut_time.time));
                if let Some(time) = self.playback_marker {
                    plot_ui.vline(VLine::new(time.time));
                }
                plot_pos = click_pos.map(|click_pos| plot_ui.plot_from_screen(click_pos));
            })
            .response;
        plot_pos
            .map(|plot_pos| {
                let absolute_time =
                    AudioTime::from_time_same_spec(plot_pos.x, self.excerpt.excerpt.start);
                self.excerpt.excerpt.get_relative_time(absolute_time)
            })
            .filter(|_| response.dragged())
    }
}
