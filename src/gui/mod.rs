mod config;
mod cutting_thread;
mod playback;
mod plot;
mod session_manager;

use std::path::Path;

use eframe::egui::Button;
use eframe::egui::Color32;
use eframe::egui::Label;
use eframe::egui::Layout;
use eframe::egui::Response;
use eframe::egui::RichText;
use eframe::egui::TextStyle;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use eframe::egui::{self};
use eframe::emath::Align;
use eframe::App;
use eframe::Frame;

use self::cutting_thread::CuttingThreadHandle;
use self::playback::play_excerpt;
use self::playback::PlaybackThreadHandle;
use self::plot::ExcerptPlot;
use crate::audio_time::AudioTime;
use crate::cut::CutInfo;
use crate::excerpt_collection::ExcerptCollection;
use crate::gui::session_manager::SessionIdentifier;
use crate::gui::session_manager::SessionManager;
use crate::recording::recording_thread_handle_status::RecordingThreadHandleStatus;
use crate::run_args::RunArgs;
use crate::service_config::Service;
use crate::service_config::ServiceConfig;
use crate::sink_type::SinkType;
use crate::song::format_title;
use crate::song::Song;

#[derive(PartialEq, Eq, Copy, Clone)]
struct SongIdentifier {
    song_index: usize,
}

pub struct StriputaryGui {
    service: Service,
    sink_type: SinkType,
    collection: Option<ExcerptCollection>,
    plots: Vec<ExcerptPlot>,
    scroll_position: usize,
    cut_thread: CuttingThreadHandle,
    record_thread: RecordingThreadHandleStatus,
    current_playback: Option<(SongIdentifier, PlaybackThreadHandle)>,
    last_touched_song: Option<SongIdentifier>,
    should_repaint: bool,
    session_manager: SessionManager,
}

impl StriputaryGui {
    pub fn new(dir: &Path, service: Service, sink_type: SinkType) -> Self {
        let session_manager = SessionManager::new(dir);
        let mut gui = Self {
            service,
            collection: None,
            plots: vec![],
            scroll_position: 0,
            cut_thread: CuttingThreadHandle::default(),
            record_thread: RecordingThreadHandleStatus::new_stopped(),
            current_playback: None,
            last_touched_song: None,
            should_repaint: false,
            session_manager,
            sink_type,
        };
        gui.load_selected_session();
        gui
    }

    fn cut_songs(&self) {
        match self.collection {
            Some(ref collection) => {
                let cut_info = self.get_cut_info(collection);
                self.cut_thread.send_cut_infos(cut_info);
            }
            None => {}
        }
    }

    fn get_cut_info(&self, collection: &ExcerptCollection) -> Vec<CutInfo> {
        let mut cut_info: Vec<CutInfo> = vec![];
        for (i, (plot_start, plot_end)) in self.plots.iter().zip(self.plots[1..].iter()).enumerate()
        {
            let song = plot_start.excerpt.song_after.as_ref().unwrap();
            cut_info.push(CutInfo::new(
                &collection.session,
                song.clone(),
                plot_start.cut_time,
                plot_end.cut_time,
                i,
            ));
        }
        cut_info
    }

    fn mark_cut_songs(&mut self) {
        let cut_songs = self.cut_thread.get_cut_songs();
        for song in cut_songs {
            for plot in self.plots.iter_mut() {
                plot.mark_cut(song);
            }
        }
    }

    fn play_last_touched_song(&mut self) {
        if let Some(last_touched) = self.last_touched_song {
            let plot = &self.plots[last_touched.song_index];
            let excerpt = &plot.excerpt.excerpt;
            if let Some((_, ref thread)) = self.current_playback {
                thread.shut_down();
            }
            self.current_playback = Some((
                last_touched,
                play_excerpt(excerpt, excerpt.get_relative_time(plot.cut_time)),
            ));
        }
    }

    fn start_recording(&mut self) {
        self.session_manager.select_new();
        self.load_selected_session();
        if !self.record_thread.is_running() {
            if let Some(ref run_args) = self.get_run_args() {
                self.record_thread = RecordingThreadHandleStatus::new_running(run_args);
            }
        }
    }

    fn get_run_args(&self) -> Option<RunArgs> {
        let service_config = ServiceConfig::from_service(self.service).unwrap();
        Some(RunArgs {
            session_dir: self.session_manager.get_currently_selected()?,
            service_config: service_config.clone(),
            sink_type: self.sink_type.clone(),
        })
    }

    fn select_session(&mut self, identifier: SessionIdentifier) {
        self.session_manager.select(identifier);
        self.load_selected_session();
    }

    fn load_selected_session(&mut self) {
        self.collection = self.session_manager.get_currently_selected_collection();
        if let Some(ref collection) = self.collection {
            self.plots = self.get_plots(collection);
        }
    }

    fn scroll(&mut self, diff: i32) {
        let num_plots = self
            .collection
            .as_ref()
            .map(|collection| collection.excerpts.len())
            .unwrap_or(0);
        self.scroll_position = (self.scroll_position as i32 + diff)
            .min(num_plots as i32 - config::MIN_NUM_PLOTS_SHOWN)
            .max(0) as usize;
    }

    fn add_large_button(&self, ui: &mut Ui, name: &str) -> Response {
        ui.add_sized(
            Vec2::new(config::CUT_BUTTON_SIZE_X, config::CUT_BUTTON_SIZE_Y),
            Button::new(RichText::new(name).text_style(TextStyle::Heading)),
        )
    }

    fn add_side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("side_bar")
            .resizable(false)
            .min_width(config::MIN_SIDE_BAR_WIDTH)
            .show(ctx, |ui| {
                let cut_button = self.add_large_button(ui, "Cut all songs");
                if cut_button.clicked() || ctx.input().key_pressed(config::CUT_KEY) {
                    self.cut_songs();
                }
                self.add_dir_selection_bar(ui);
            });
    }

    fn add_dir_selection_bar(&mut self, ui: &mut Ui) {
        self.add_record_button_or_error_message(ui);
        ui.add(Label::new(
            RichText::new("Previous sessions:").text_style(TextStyle::Heading),
        ));
        let dirs_with_indices: Vec<_> = self
            .session_manager
            .iter_relative_paths_with_indices()
            .collect();
        for (i, dir_name) in dirs_with_indices.iter() {
            let mut button_text = RichText::new(dir_name).text_style(TextStyle::Heading);
            let button = if self.session_manager.is_currently_selected(i) {
                button_text = button_text.color(config::SELECTED_TEXT_COLOR);
                Button::new(button_text).fill(config::SELECTED_FILL_COLOR)
            } else {
                Button::new(button_text)
            };
            if ui.add(button).clicked() {
                self.select_session(*i);
            }
        }
    }

    fn add_record_button_or_error_message(&mut self, ui: &mut Ui) {
        if !self.record_thread.is_running() {
            self.add_record_button(ui);
        }
        if let RecordingThreadHandleStatus::Failed(ref error) = self.record_thread {
            self.add_recording_thread_error_message(ui, error);
        }
    }

    fn add_record_button(&mut self, ui: &mut Ui) {
        let record_button = self.add_large_button(ui, "Record new session");
        if record_button.clicked() {
            self.start_recording();
        }
    }

    fn add_recording_thread_error_message(&self, ui: &mut Ui, error: &anyhow::Error) {
        let label = Label::new(RichText::new(error.to_string()).color(Color32::RED));
        ui.add(label);
    }

    fn add_labels_for_recorded_songs(&self, ui: &mut Ui) {
        let songs = self.record_thread.get_songs();
        for song in songs.iter().rev() {
            let label = Label::new(RichText::new(song.to_string()));
            ui.add(label);
        }
    }

    fn enumerate_visible_plots(
        &self,
        num_shown: i32,
    ) -> impl Iterator<Item = (usize, &ExcerptPlot)> {
        let min = self.scroll_position.min(self.plots.len());
        let max = (self.scroll_position + num_shown as usize).min(self.plots.len());
        let slice = &self.plots[min..max];
        slice
            .iter()
            .enumerate()
            .map(move |(i, s)| (i + self.scroll_position, s))
    }

    fn move_all_markers_after(&mut self, clicked_song: SongIdentifier, offset: AudioTime) {
        for plot in self.plots.iter_mut() {
            if plot.excerpt.num >= clicked_song.song_index {
                plot.move_marker_to_offset(offset);
            }
        }
    }

    fn add_plot_labels(ui: &mut Ui, plot: &ExcerptPlot) {
        ui.horizontal(|ui| {
            add_plot_label(
                ui,
                plot.excerpt.song_before.as_ref(),
                plot.finished_cutting_song_before,
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                add_plot_label(
                    ui,
                    plot.excerpt.song_after.as_ref(),
                    plot.finished_cutting_song_after,
                );
            });
        });
    }

    fn set_playback_marker_and_return_finished_state(
        plot: &mut ExcerptPlot,
        current_playback: &PlaybackThreadHandle,
    ) -> bool {
        let playback_time_relative = current_playback.get_elapsed_audio_time();
        let playback_time_absolute = plot.excerpt.excerpt.start + playback_time_relative;
        if playback_time_absolute < plot.excerpt.excerpt.end {
            plot.show_playback_marker_at(playback_time_absolute);
            return false;
        } else {
            return true;
        }
    }

    fn add_central_panel(&mut self, ctx: &egui::Context) {
        let mouse_pos = ctx.input().pointer.interact_pos();
        let mut clicked_song_and_offset: Option<(SongIdentifier, AudioTime)> = None;
        let panel_height = ctx.used_size().y;
        let num_plots_shown = (panel_height / config::PLOT_HEIGHT).ceil() as i32;
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.record_thread.is_running() {
                self.add_labels_for_recorded_songs(ui);
            } else {
                for (plot_song, plot) in self
                    .enumerate_visible_plots(num_plots_shown)
                    .map(|(song_index, plot)| (SongIdentifier { song_index }, plot))
                {
                    Self::add_plot_labels(ui, plot);
                    let offset = plot.show_and_get_offset(plot_song.song_index, ui, mouse_pos);
                    if let Some(offset) = offset {
                        clicked_song_and_offset = Some((plot_song, offset));
                    };
                }
            }
        });
        if let Some((clicked_song, offset)) = clicked_song_and_offset {
            self.last_touched_song = Some(clicked_song);
            self.move_all_markers_after(clicked_song, offset);
        }
    }

    fn keyboard_control(&mut self, ctx: &egui::Context) {
        if ctx.input().key_pressed(config::SCROLL_UP_KEY) {
            self.scroll(-1);
        }
        if ctx.input().key_pressed(config::SCROLL_DOWN_KEY) {
            self.scroll(1);
        }
        if ctx.input().key_pressed(config::PLAYBACK_KEY) {
            self.play_last_touched_song();
        }
    }

    fn handle_playback_markers(&mut self) {
        for mut plot in self.plots.iter_mut() {
            plot.hide_playback_marker();
            if let Some((playback_song, ref current_playback_handle)) = self.current_playback {
                if playback_song.song_index == plot.excerpt.num {
                    self.should_repaint = !Self::set_playback_marker_and_return_finished_state(
                        &mut plot,
                        current_playback_handle,
                    );
                }
            }
        }
    }

    fn get_plots(&self, collection: &ExcerptCollection) -> Vec<ExcerptPlot> {
        collection
            .excerpts
            .iter()
            .map(|excerpt| {
                ExcerptPlot::new(
                    excerpt.clone(),
                    excerpt
                        .excerpt
                        .get_absolute_time_from_time_offset(collection.offset_guess),
                )
            })
            .collect()
    }
}

impl App for StriputaryGui {
    fn update(&mut self, ctx: &egui::Context, _: &mut Frame) {
        self.record_thread.update();
        self.add_side_panel(ctx);
        self.handle_playback_markers();
        self.add_central_panel(ctx);
        self.mark_cut_songs();
        if self.should_repaint {
            ctx.request_repaint();
            self.should_repaint = false;
        }
        self.keyboard_control(ctx);
    }
}

pub fn get_label_color(finished_cutting: bool) -> Color32 {
    match finished_cutting {
        true => config::CUT_LABEL_COLOR,
        false => config::UNCUT_LABEL_COLOR,
    }
}

fn add_plot_label(ui: &mut Ui, song: Option<&Song>, finished_cutting: bool) {
    let color = get_label_color(finished_cutting);
    if let Some(ref song) = song {
        ui.add(Label::new(
            RichText::new(format_title(&song.title)).color(color),
        ));
    }
}
