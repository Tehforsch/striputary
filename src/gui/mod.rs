mod config;
mod cutting_thread;
mod plot;

use crate::{cut::CutInfo, excerpt_collection::{ExcerptCollection, NamedExcerpt}, excerpt_collections::ExcerptCollections, song::Song};
use eframe::{egui, epi};

use self::{cutting_thread::CuttingThreadHandle, plot::ExcerptPlot};

pub struct StriputaryGui {
    collections: ExcerptCollections,
    plots: Vec<ExcerptPlot>,
    thread: CuttingThreadHandle,
}

impl StriputaryGui {
    pub fn new(collections: ExcerptCollections) -> Self {
        let collection = collections.get_selected();
        let plots = StriputaryGui::get_plots(collection);
        let thread = CuttingThreadHandle::default();
        Self {
            collections,
            plots,
            thread,
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

    fn cut_songs(&self) {
        let cut_infos = self.get_cut_info();
        self.thread.send_cut_infos(cut_infos);
    }

    fn get_cut_info(&self) -> Vec<CutInfo> {
        let collection = self.collections.get_selected();
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
        let cut_songs = self.thread.get_cut_songs();
        for song in cut_songs {
            for plot in self.plots.iter_mut() {
                if let Some(ref song_before) = plot.excerpt.song_before {
                    if song_before == song {
                        plot.finished_cutting_song_before = true;
                    }
                }
                if let Some(ref song_after) = plot.excerpt.song_after {
                    if song_after == song {
                        plot.finished_cutting_song_after = true;
                    }
                }
            }
        }
    }

    fn select(&mut self, selection: usize) {
        self.collections.select(selection);
        self.plots = StriputaryGui::get_plots(self.collections.get_selected());
    }
}

impl epi::App for StriputaryGui {
    fn name(&self) -> &str {
        "Striputary"
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                let mut selection: Option<usize> = None;
                for (i, collection) in self.collections.enumerate() {
                    if ui.button(collection.name()).clicked() {
                        selection = Some(i);
                    }
                }
                if let Some(selection) = selection {
                    self.select(selection);
                }
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            if ui.button("Cut").clicked() {
                self.cut_songs();
            }
        });

        self.mark_cut_songs();

        egui::CentralPanel::default().show(ctx, |ui| {
            for plot in self.plots.iter_mut() {
                // if let Some(ref song) = plot.excerpt.song {
                    // ui.label(format!("{} - {}", song.artist, song.title));
                // }
                ui.add(plot);
            }
            egui::warn_if_debug_build(ui);
        });
    }
}
