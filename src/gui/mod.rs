mod config;
mod cutting_thread;
mod playback;
mod plot;
mod session_dir_manager;

use std::path::{Path, PathBuf};

use crate::{
    cut::{get_excerpt_collection, CutInfo},
    excerpt_collection::ExcerptCollection,
    gui::session_dir_manager::SessionDirManager,
    recording::recording_thread_handle_status::RecordingThreadHandleStatus,
    recording_session::{load_sessions, RecordingSession},
    run_args::RunArgs,
    service_config::ServiceConfig,
    song::Song,
};

use eframe::{
    egui::{self, Button, Color32, Label, Layout, Pos2, Response, TextStyle, Ui},
    epi,
};

use self::{
    cutting_thread::CuttingThreadHandle,
    playback::{play_excerpt, PlaybackThreadHandle},
    plot::ExcerptPlot,
};

type CollectionIdentifier = usize;

#[derive(PartialEq, Eq, Copy, Clone)]
struct SongIdentifier {
    song_index: usize,
    collection_index: CollectionIdentifier,
}

pub struct StriputaryGui {
    service_config: ServiceConfig,
    collections: Vec<ExcerptCollection>,
    plots: Vec<ExcerptPlot>,
    cut_thread: CuttingThreadHandle,
    record_thread: RecordingThreadHandleStatus,
    current_playback: Option<(SongIdentifier, PlaybackThreadHandle)>,
    last_touched_song: Option<SongIdentifier>,
    selected_collection: CollectionIdentifier,
    should_repaint: bool,
    session_dir_manager: SessionDirManager,
}

impl StriputaryGui {
    pub fn new(dir: &Path, service_config: ServiceConfig) -> Self {
        let session_dir_manager = SessionDirManager::new(dir);
        let mut gui = Self {
            service_config,
            collections: vec![],
            plots: vec![],
            cut_thread: CuttingThreadHandle::default(),
            record_thread: RecordingThreadHandleStatus::new_stopped(),
            current_playback: None,
            last_touched_song: None,
            selected_collection: 0,
            should_repaint: false,
            session_dir_manager,
        };
        gui.load_selected_session();
        gui
    }

    fn cut_songs(&self) {
        let cut_infos = self.get_cut_info();
        self.cut_thread.send_cut_infos(cut_infos);
    }

    fn get_cut_info(&self) -> Vec<CutInfo> {
        let collection = self.get_selected_collection().unwrap();
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
                if let Some(ref song_before) = plot.excerpt.song_before {
                    if song_before == song {
                        if !plot.finished_cutting_song_before {
                            plot.finished_cutting_song_before = true;
                            self.should_repaint = true;
                        }
                    }
                }
                if let Some(ref song_after) = plot.excerpt.song_after {
                    if song_after == song {
                        if !plot.finished_cutting_song_after {
                            plot.finished_cutting_song_after = true;
                            self.should_repaint = true;
                        }
                    }
                }
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
        if !self.record_thread.is_running() {
            self.record_thread = RecordingThreadHandleStatus::new_running(&self.get_run_args());
        }
    }

    fn get_run_args(&self) -> RunArgs {
        RunArgs {
            session_dir: self.session_dir_manager.get_currently_selected(),
            service_config: self.service_config.clone(),
        }
    }

    fn select_session_folder_by_index(&mut self, index: usize) {
        self.session_dir_manager.select(index);
        self.load_selected_session();
    }

    fn load_selected_session(&mut self) {
        self.collections = self
            .session_dir_manager
            .get_currently_selected_collections();
        self.try_select_collection(0);
    }

    fn try_select_collection(&mut self, selection: CollectionIdentifier) {
        self.selected_collection = selection;
        self.plots = match self.get_selected_collection() {
            None => vec![],
            Some(collection) => get_plots(collection),
        }
    }

    pub fn get_selected_collection(&self) -> Option<&ExcerptCollection> {
        self.collections.get(self.selected_collection)
    }

    pub fn select_next_collection(&mut self) {
        if self.selected_collection == self.collections.len() - 1 {
            return;
        }
        self.try_select_collection(self.selected_collection + 1);
    }

    pub fn select_previous_collection(&mut self) {
        if self.selected_collection == 0 {
            return;
        }
        self.try_select_collection(self.selected_collection - 1);
    }

    fn add_top_bar(&mut self, ctx: &egui::CtxRef) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.columns(self.collections.len(), |columns| {
                let mut selection: Option<usize> = None;
                for (i, collection) in self.collections.iter().enumerate() {
                    let is_selected = self.selected_collection == i;
                    let button = add_collection_button(&mut columns[i], is_selected, collection);
                    if button.clicked() {
                        selection = Some(i);
                    }
                }
                if let Some(selection) = selection {
                    self.try_select_collection(selection);
                }
            });
        });
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
                self.add_record_button_or_error_message(ui);
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
                ui.add(Label::new("Previous sessions:").text_style(TextStyle::Heading));
                let dirs_with_indices: Vec<_> = self
                    .session_dir_manager
                    .iter_relative_paths()
                    .enumerate()
                    .collect();
                for (i, dir_name) in dirs_with_indices.iter() {
                    let button = Button::new(dir_name).text_style(TextStyle::Heading);
                    if ui.add(button).clicked() {
                        self.select_session_folder_by_index(*i);
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
        let record_button = self.add_large_button(ui, "Record");
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
        let collection_index = self.selected_collection;
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut clicked_pos: Option<Pos2> = None;
            for (plot_song, plot) in self.plots.iter_mut().enumerate().map(|(song_index, plot)| {
                (
                    SongIdentifier {
                        song_index,
                        collection_index,
                    },
                    plot,
                )
            }) {
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
}

impl epi::App for StriputaryGui {
    fn name(&self) -> &str {
        "Striputary"
    }

    fn update(&mut self, ctx: &egui::CtxRef, _: &mut epi::Frame<'_>) {
        self.record_thread.update();
        self.add_top_bar(ctx);
        self.add_dir_selection_bar(ctx);
        self.add_side_bar(ctx);
        self.add_central_panel(ctx);
        self.mark_cut_songs();
        if ctx.input().key_pressed(config::SELECT_NEXT_KEY) {
            self.select_next_collection();
        }
        if ctx.input().key_pressed(config::SELECT_PREVIOUS_KEY) {
            self.select_previous_collection();
        }
        if self.should_repaint {
            ctx.request_repaint();
            self.should_repaint = false;
        }
    }
}

fn get_plots(collection: &ExcerptCollection) -> Vec<ExcerptPlot> {
    collection
        .iter_excerpts()
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

pub fn get_label_color(finished_cutting: bool) -> Color32 {
    match finished_cutting {
        true => config::CUT_LABEL_COLOR,
        false => config::UNCUT_LABEL_COLOR,
    }
}

fn add_collection_button(ui: &mut Ui, selected: bool, collection: &ExcerptCollection) -> Response {
    let label = collection.name();
    let mut button = Button::new(label).text_style(TextStyle::Heading);
    if selected {
        button = button.fill(config::SELECTED_COLLECTION_FILL_COLOR);
        button = button.text_color(config::SELECTED_COLLECTION_TEXT_COLOR);
    }
    ui.add(button)
}

fn add_plot_label(ui: &mut Ui, song: Option<&Song>, finished_cutting: bool) {
    let color = get_label_color(finished_cutting);
    if let Some(ref song) = song {
        ui.add(Label::new(format!("{}", song.title)).text_color(color));
    }
}
