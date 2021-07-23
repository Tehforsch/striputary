mod config;
mod cutting_thread;
mod graphics;
mod input;
mod offset_marker;

use std::thread;

use self::{
    cutting_thread::CuttingThreadHandle,
    graphics::{
        camera_positioning_system, initialize_camera_system, marker_positioning_system,
        show_excerpts_system, spawn_offset_markers_system, text_positioning_system,
        z_layering_system, ScrollPosition,
    },
    input::{
        exit_system, move_markers_on_click_system, scrolling_input_system,
        track_mouse_position_system, MousePosition,
    },
    offset_marker::PositionMarker,
};
use crate::{
    audio_excerpt::AudioExcerpt,
    config::NUM_OFFSETS_TO_TRY,
    cut::{cut_song, get_named_excerpts, CutInfo, NamedExcerpt},
    recording_session::RecordingSession,
    song::Song,
};
use bevy::prelude::*;
use bevy_prototype_lyon::plugin::ShapePlugin;

pub fn run(session: RecordingSession) {
    App::build()
        .add_plugins(DefaultPlugins)
        .init_resource::<MousePosition>()
        .insert_resource(session)
        .insert_resource(ScrollPosition(0))
        .init_non_send_resource::<CuttingThreadHandle>()
        // .insert_resource(Msaa { samples: 8 })
        .add_plugin(ShapePlugin)
        .add_startup_system(initialize_camera_system.system())
        .add_startup_system(add_excerpts_system.system())
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
        .run();
}

fn add_excerpts_system(mut commands: Commands, session: Res<RecordingSession>) {
    let excerpts = get_named_excerpts(&session);
    for (i, excerpt) in excerpts.into_iter().enumerate() {
        commands.spawn().insert(excerpt);
        commands.spawn().insert(PositionMarker::new(i));
    }
}

fn get_volume_data(excerpt: &AudioExcerpt) -> Vec<f64> {
    let width = excerpt.end.time - excerpt.start.time;
    let step_size = width / NUM_OFFSETS_TO_TRY as f64;
    let times = (1..NUM_OFFSETS_TO_TRY).map(|x| excerpt.start.time + (x as f64) * step_size);
    times.map(|time| excerpt.get_volume_at(time)).collect()
}

fn start_cut_system(
    keyboard_input: Res<Input<KeyCode>>,
    session: Res<RecordingSession>,
    positions: Query<&PositionMarker>,
    excerpts: Query<&NamedExcerpt>,
    cutting_thread: NonSend<CuttingThreadHandle>,
) {
    for key in keyboard_input.get_just_pressed() {
        if let KeyCode::Return = key {
            let cut_infos = get_cut_info(&session, &positions, &excerpts);
            let cloned_session = session.clone();
            cutting_thread.send_cut_infos(cut_infos);
        }
    }
}

fn get_cut_info(
    session: &RecordingSession,
    positions: &Query<&PositionMarker>,
    excerpts: &Query<&NamedExcerpt>,
) -> Vec<CutInfo> {
    let mut markers: Vec<&PositionMarker> = positions.iter().collect();
    markers.sort_by_key(|marker| marker.num);
    let mut excerpts: Vec<&NamedExcerpt> = excerpts.iter().collect();
    excerpts.sort_by_key(|excerpt| excerpt.num);
    let mut cut_info: Vec<CutInfo> = vec![];
    for ((excerpt_start, excerpt_end), (marker_start, marker_end)) in excerpts
        .iter()
        .zip(excerpts[1..].iter())
        .zip(markers.iter().zip(markers[1..].iter()))
    {
        let song = &session.songs[marker_start.num];
        let start_time = marker_start.get_audio_time(&excerpt_start.excerpt);
        let end_time = marker_end.get_audio_time(&excerpt_end.excerpt);
        cut_info.push(CutInfo::new(session, song.clone(), start_time, end_time));
    }
    cut_info
}
