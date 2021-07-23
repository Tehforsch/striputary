mod config;
mod cutting_thread;
mod excerpt_view;
mod graphics;
mod input;
mod offset_marker;

use self::{
    cutting_thread::CuttingThreadHandle,
    excerpt_view::ExcerptView,
    graphics::{
        camera_positioning_system, initialize_camera_system, marker_positioning_system,
        show_excerpts_system, spawn_offset_markers_system, text_positioning_system,
        z_layering_system, ScrollPosition,
    },
    input::{
        collection_selection_input, exit_system, move_markers_on_click_system,
        scrolling_input_system, track_mouse_position_system, MousePosition,
    },
    offset_marker::PositionMarker,
};
use crate::{
    cut::{get_excerpt_collection, CutInfo},
    excerpt_collection::{ExcerptCollection, NamedExcerpt},
    excerpt_collections::ExcerptCollections,
    recording_session::RecordingSession,
};
use bevy::prelude::*;
use bevy_prototype_lyon::plugin::ShapePlugin;

pub struct ReadCollectionEvent;

pub fn run(sessions: Vec<RecordingSession>) {
    let collections = ExcerptCollections::new(
        sessions
            .into_iter()
            .map(|session| get_excerpt_collection(session))
            .collect(),
    );
    App::build()
        .add_plugins(DefaultPlugins)
        .init_resource::<MousePosition>()
        .add_event::<ReadCollectionEvent>()
        .insert_resource(collections)
        .insert_resource(ScrollPosition(0))
        .init_non_send_resource::<CuttingThreadHandle>()
        // .insert_resource(Msaa { samples: 8 })
        .add_plugin(ShapePlugin)
        .add_startup_system(initialize_camera_system.system())
        .add_startup_system(load_first_collection.system())
        .add_system(add_excerpts_and_markers_system.system())
        .add_system(remove_excerpts_and_markers_system.system())
        .add_system(show_excerpts_system.system())
        .add_system(text_positioning_system.system())
        .add_system(camera_positioning_system.system())
        .add_system(z_layering_system.system())
        .add_system(scrolling_input_system.system())
        .add_system(spawn_offset_markers_system.system())
        .add_system(exit_system.system())
        .add_system(start_cut_system.system())
        .add_system(track_mouse_position_system.system())
        .add_system(move_markers_on_click_system.system())
        .add_system(marker_positioning_system.system())
        .add_system(collection_selection_input.system())
        .run();
}

fn load_first_collection(
    mut read_collection_events: EventWriter<ReadCollectionEvent>) {
    read_collection_events.send(ReadCollectionEvent);
}

fn remove_excerpts_and_markers_system(
    mut commands: Commands,
    mut read_collection_events: EventReader<ReadCollectionEvent>,
    excerpt_views: Query<Entity, With<ExcerptView>>,
) {
    for _ in read_collection_events.iter() {
        for entity in excerpt_views.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn add_excerpts_and_markers_system(
    mut commands: Commands,
    collections: Res<ExcerptCollections>,
    mut read_collection_events: EventReader<ReadCollectionEvent>,
) {
    for _ in read_collection_events.iter() {
        let collection = collections.get_selected();
        for (i, excerpt) in collection.iter_excerpts().enumerate() {
            let relative_progress = excerpt
                .excerpt
                .get_relative_progress_from_time_offset(collection.offset_guess);
            let marker_id = commands
                .spawn()
                .insert(PositionMarker::new(i, relative_progress))
                .id();
            commands
                .spawn()
                .insert(ExcerptView::new(i))
                .push_children(&[marker_id]);
        }
    }
}

fn start_cut_system(
    keyboard_input: Res<Input<KeyCode>>,
    collections: Res<ExcerptCollections>,
    positions: Query<&PositionMarker>,
    cutting_thread: NonSend<CuttingThreadHandle>,
) {
    for key in keyboard_input.get_just_pressed() {
        if let KeyCode::Return = key {
            let cut_infos = get_cut_info(&collections.get_selected(), &positions);
            cutting_thread.send_cut_infos(cut_infos);
        }
    }
}

fn get_cut_info(
    collection: &ExcerptCollection,
    positions: &Query<&PositionMarker>,
) -> Vec<CutInfo> {
    let mut markers: Vec<&PositionMarker> = positions.iter().collect();
    markers.sort_by_key(|marker| marker.num);
    let excerpts: Vec<&NamedExcerpt> = collection.iter_excerpts().collect();
    let mut cut_info: Vec<CutInfo> = vec![];
    for ((excerpt_start, excerpt_end), (marker_start, marker_end)) in excerpts
        .iter()
        .zip(excerpts[1..].iter())
        .zip(markers.iter().zip(markers[1..].iter()))
    {
        let song = &excerpt_end.song;
        let start_time = marker_start.get_audio_time(&excerpt_start.excerpt);
        let end_time = marker_end.get_audio_time(&excerpt_end.excerpt);
        cut_info.push(CutInfo::new(
            &collection.session,
            song.clone(),
            start_time,
            end_time,
        ));
    }
    cut_info
}
