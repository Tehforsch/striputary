mod config;
mod cutting_thread;
mod playback;
mod plot;

use crate::{
    cut::CutInfo,
    excerpt_collection::ExcerptCollection,
    record::{FallibleRecordingThreadHandle, RecordingThreadHandle, RecordingThreadHandleStatus},
    run_args::RunArgs,
    song::Song,
};

use anyhow::Result;
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
    run_args: RunArgs,
    collections: Vec<ExcerptCollection>,
    plots: Vec<ExcerptPlot>,
    cut_thread: CuttingThreadHandle,
    record_thread: FallibleRecordingThreadHandle,
    current_playback: Option<(SongIdentifier, PlaybackThreadHandle)>,
    last_touched_song: Option<SongIdentifier>,
    selected_collection: CollectionIdentifier,
    should_repaint: bool,
}

impl StriputaryGui {
    pub fn new(run_args: &RunArgs, collections: Vec<ExcerptCollection>) -> Self {
        let cut_thread = CuttingThreadHandle::default();
        let mut gui = Self {
            run_args: run_args.clone(),
            collections,
            plots: vec![],
            cut_thread,
            record_thread: FallibleRecordingThreadHandle::new_stopped(),
            current_playback: None,
            last_touched_song: None,
            selected_collection: 0,
            should_repaint: false,
        };
        gui.try_select(0);
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
            self.record_thread = FallibleRecordingThreadHandle::new_running(&self.run_args);
        }
    }

    fn try_select(&mut self, selection: CollectionIdentifier) {
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
        self.try_select(self.selected_collection + 1);
    }

    pub fn select_previous_collection(&mut self) {
        if self.selected_collection == 0 {
            return;
        }
        self.try_select(self.selected_collection - 1);
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
                    self.try_select(selection);
                }
            });
        });
    }

    fn add_large_button(&self, ui: &mut Ui, name: &str) -> Response {
        ui.add(Button::new(name).text_style(TextStyle::Heading))
    }

    fn add_side_bar(&mut self, ctx: &egui::CtxRef) {
        egui::SidePanel::left("side_panel")
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

    fn add_record_button_or_error_message(&mut self, ui: &mut Ui) {
        if !self.record_thread.is_running() {
            self.add_record_button(ui);
        }
        if let RecordingThreadHandleStatus::Failed(ref error) = self.record_thread.status {
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
