mod config;
mod cutting_thread;
mod playback;
mod plot;
mod session_manager;

use std::path::Path;

use crate::cut::CutInfo;
use crate::excerpt_collection::ExcerptCollection;
use crate::gui::session_manager::SessionIdentifier;
use crate::gui::session_manager::SessionManager;
use crate::recording::recording_thread_handle_status::RecordingThreadHandleStatus;
use crate::run_args::RunArgs;
use crate::service_config::ServiceConfig;
use crate::song::Song;

use eframe::egui::Button;
use eframe::egui::Color32;
use eframe::egui::Label;
use eframe::egui::Layout;
use eframe::egui::Pos2;
use eframe::egui::Response;
use eframe::egui::TextStyle;
use eframe::egui::Ui;
use eframe::egui::{self};
use eframe::epi;

use self::cutting_thread::CuttingThreadHandle;
use self::playback::play_excerpt;
use self::playback::PlaybackThreadHandle;
use self::plot::ExcerptPlot;

#[derive(PartialEq, Eq, Copy, Clone)]
struct SongIdentifier {
    song_index: usize,
}

pub struct StriputaryGui {
    service_config: ServiceConfig,
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
    pub fn new(dir: &Path, service_config: ServiceConfig) -> Self {
        let session_manager = SessionManager::new(dir);
        let mut gui = Self {
            service_config,
            collection: None,
            plots: vec![],
            scroll_position: 0,
            cut_thread: CuttingThreadHandle::default(),
            record_thread: RecordingThreadHandleStatus::new_stopped(),
            current_playback: None,
            last_touched_song: None,
            should_repaint: false,
            session_manager,
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
        for (plot_start, plot_end) in self.plots.iter().zip(self.plots[1..].iter()) {
            let song = plot_start.excerpt.song_after.as_ref().unwrap();
            cut_info.push(CutInfo::new(
                &collection.session,
                song.clone(),
                plot_start.cut_time,
                plot_end.cut_time,
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
            self.record_thread = RecordingThreadHandleStatus::new_running(&self.get_run_args());
        }
    }

    fn get_run_args(&self) -> RunArgs {
        RunArgs {
            session_dir: self.session_manager.get_currently_selected(),
            service_config: self.service_config.clone(),
        }
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
        let max_num_plots = self
            .collection
            .as_ref()
            .map(|collection| collection.excerpts.len())
            .unwrap_or(0);
        self.scroll_position = (self.scroll_position as i32 + diff)
            .min(max_num_plots as i32 - config::NUM_PLOTS_TO_SHOW as i32)
            .max(0) as usize;
    }

    fn add_large_button(&self, ui: &mut Ui, name: &str) -> Response {
        ui.add(Button::new(name).text_style(TextStyle::Heading))
    }

    fn add_side_bar(&mut self, ctx: &egui::CtxRef) {
        egui::SidePanel::left("buttons")
            .resizable(false)
            .show(ctx, |ui| {
                let cut_button = self.add_large_button(ui, "Cut");
                let playback_button = self.add_large_button(ui, "Playback");
                if cut_button.clicked() || ctx.input().key_pressed(config::CUT_KEY) {
                    self.cut_songs();
                }
                if playback_button.clicked() || ctx.input().key_pressed(config::PLAYBACK_KEY) {
                    self.play_last_touched_song();
                }
            });
    }

    fn add_dir_selection_bar(&mut self, ctx: &egui::CtxRef) {
        egui::SidePanel::left("dir_select")
            .resizable(false)
            .show(ctx, |ui| {
                self.add_record_button_or_error_message(ui);
                ui.add(Label::new("Previous sessions:").text_style(TextStyle::Heading));
                let dirs_with_indices: Vec<_> = self
                    .session_manager
                    .iter_relative_paths_with_indices()
                    .collect();
                for (i, dir_name) in dirs_with_indices.iter() {
                    let mut button = Button::new(dir_name).text_style(TextStyle::Heading);
                    if self.session_manager.is_currently_selected(i) {
                        button = button
                            .fill(config::SELECTED_FILL_COLOR)
                            .text_color(config::SELECTED_TEXT_COLOR);
                    }
                    if ui.add(button).clicked() {
                        self.select_session(*i);
                    }
                }
            });
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
        let label = Label::new(error.to_string()).text_color(Color32::RED);
        ui.add(label);
    }

    fn add_labels_for_recorded_songs(&self, ui: &mut Ui) {
        if self.record_thread.is_running() {
            let songs = self.record_thread.get_songs();
            for song in songs.iter().rev() {
                let mut label = Label::new(song.to_string());
                label = label.text_style(TextStyle::Heading);
                if songs
                    .last()
                    .map(|last_song| last_song == song)
                    .unwrap_or(false)
                {
                    label = label.underline();
                }
                ui.add(label);
            }
        }
    }

    fn add_central_panel(&mut self, ctx: &egui::CtxRef) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut clicked_pos: Option<Pos2> = None;
            for (plot_song, plot) in self
                .plots
                .iter_mut()
                .enumerate()
                .map(|(song_index, plot)| (SongIdentifier { song_index }, plot))
            {
                ui.horizontal(|ui| {
                    add_plot_label(
                        ui,
                        plot.excerpt.song_before.as_ref(),
                        plot.finished_cutting_song_before,
                    );
                    ui.with_layout(Layout::right_to_left(), |ui| {
                        add_plot_label(
                            ui,
                            plot.excerpt.song_after.as_ref(),
                            plot.finished_cutting_song_after,
                        );
                    });
                });
                plot.hide_playback_marker();
                if let Some((playback_song, ref current_playback)) = self.current_playback {
                    if playback_song == plot_song {
                        let playback_time_relative = current_playback.get_elapsed_audio_time();
                        let playback_time_absolute =
                            plot.excerpt.excerpt.start + playback_time_relative;
                        if playback_time_absolute < plot.excerpt.excerpt.end {
                            self.should_repaint = true;
                            plot.show_playback_marker_at(playback_time_absolute);
                        }
                    }
                }
                plot.move_cut_marker_to_pos = clicked_pos;
                let plot = ui.add(plot);
                if plot.is_pointer_button_down_on() {
                    self.last_touched_song = Some(plot_song);
                    if let Some(pos) = plot.interact_pointer_pos() {
                        clicked_pos = Some(pos);
                    }
                };
            }
            self.add_labels_for_recorded_songs(ui);
        });
    }

    fn keyboard_control(&mut self, ctx: &egui::CtxRef) {
        if ctx.input().key_pressed(config::SCROLL_UP_KEY) {
            self.scroll(-1);
        }
        if ctx.input().key_pressed(config::SCROLL_DOWN_KEY) {
            self.scroll(1);
        }
        println!("{}", self.scroll_position);
    }

    fn get_plots(&self, collection: &ExcerptCollection) -> Vec<ExcerptPlot> {
        collection.excerpts[self.scroll_position..self.scroll_position + config::NUM_PLOTS_TO_SHOW]
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

impl epi::App for StriputaryGui {
    fn name(&self) -> &str {
        "Striputary"
    }

    fn update(&mut self, ctx: &egui::CtxRef, _: &mut epi::Frame<'_>) {
        self.record_thread.update();
        self.add_dir_selection_bar(ctx);
        self.add_side_bar(ctx);
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
        ui.add(Label::new(format!("{}", song.title)).text_color(color));
    }
}
