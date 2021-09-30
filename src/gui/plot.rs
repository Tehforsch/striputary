use eframe::egui::plot::*;
use eframe::egui::*;

use crate::audio_time::AudioTime;
use crate::excerpt_collection::NamedExcerpt;

use super::config;

pub struct ExcerptPlot {
    pub excerpt: NamedExcerpt,
    pub cut_time: AudioTime,
    pub finished_cutting_song_before: bool,
    pub finished_cutting_song_after: bool,
}

impl ExcerptPlot {
    pub fn new(excerpt: NamedExcerpt, cut_time: AudioTime) -> Self {
        Self { excerpt, cut_time, finished_cutting_song_before: false, finished_cutting_song_after: false }
    }

    fn get_lines(&self) -> (Line, Line) {
        let x_values = self.excerpt.excerpt.get_sample_times();
        let y_values = self.excerpt.excerpt.get_volume_plot_data();
        let values_iter = x_values.into_iter().zip(y_values).map(|(x, y)| Value::new(x, y));
        let (values_before_cut, values_after_cut): (Vec<_>, Vec<_>) = values_iter.partition(|value| value.x < self.cut_time.time);
        (Line::new(Values::from_values(values_before_cut)), Line::new(Values::from_values(values_after_cut)))
    }

    pub fn get_line_color(&self, finished_cutting: bool) -> Color32 {
        match finished_cutting {
            true => config::CUT_LINE_COLOR,
            false => config::UNCUT_LINE_COLOR,
        }
    }

    pub fn set_offset(&mut self, click_pos: Pos2, rect: Rect) {
        let plot_begin = rect.min + (rect.center() - rect.min) * 0.05;
        let plot_width = rect.width() / 1.1;
        let relative_progress = (click_pos.x - plot_begin.x) / plot_width;
        self.cut_time = self.excerpt.excerpt.get_absolute_time_by_relative_progress(relative_progress as f64);
    }
}

impl Widget for &mut ExcerptPlot {
    fn ui(self, ui: &mut Ui) -> Response {
        let (line_before, line_after) = self.get_lines();
        let plot = Plot::new("volume")
            .line(line_before.color(self.get_line_color(self.finished_cutting_song_before)))
            .line(line_after.color(self.get_line_color(self.finished_cutting_song_after)))
            .legend(Legend::default())
            .view_aspect(config::PLOT_ASPECT)
            .show_x(false)
            .show_y(false)
            .allow_drag(false)
            .allow_zoom(false)
            .vline(VLine::new(self.cut_time.time))
            .show_background(false)
            ;
        let response = ui.add(plot);
        if response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                self.set_offset(pos, response.rect);
            }
        }
        response
    }
}
