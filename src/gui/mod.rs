mod config;
mod graphics;
mod input;
mod offset_marker;

use self::{graphics::{ScrollPosition, camera_positioning_system, initialize_camera_system, marker_positioning_system, show_excerpts_system, spawn_offset_markers_system, text_positioning_system, z_layering_system}, input::{MousePosition, exit_system, move_markers_on_click_system, scrolling_input_system, track_mouse_position_system}, offset_marker::OffsetMarker};
use crate::{
    audio_excerpt::AudioExcerpt,
    config::NUM_OFFSETS_TO_TRY,
    cut::{cut_song, get_named_excerpts},
    recording_session::RecordingSession,
};
use bevy::prelude::*;
use bevy_prototype_lyon::plugin::ShapePlugin;

pub fn run(session: RecordingSession) {
    App::build()
        .add_plugins(DefaultPlugins)
        .init_resource::<MousePosition>()
        .insert_resource(session)
        .insert_resource(ScrollPosition(0))
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
        .add_system(cut_system.system())
        .add_system(track_mouse_position_system.system())
        .add_system(move_markers_on_click_system.system())
        .add_system(marker_positioning_system.system())
        .run();
}

fn add_excerpts_system(mut commands: Commands, session: Res<RecordingSession>) {
    let excerpts = get_named_excerpts(&session);
    for (i, excerpt) in excerpts.into_iter().enumerate() {
        commands.spawn().insert(excerpt);
        commands.spawn().insert(OffsetMarker { num: i, pos: 0.0 });
    }
}

fn get_volume_data(excerpt: &AudioExcerpt) -> Vec<f64> {
    let width = excerpt.end.time - excerpt.start.time;
    let step_size = width / NUM_OFFSETS_TO_TRY as f64;
    let times = (1..NUM_OFFSETS_TO_TRY).map(|x| excerpt.start.time + (x as f64) * step_size);
    times.map(|time| excerpt.get_volume_at(time)).collect()
}

fn cut_system(
    keyboard_input: Res<Input<KeyCode>>,
    session: Res<RecordingSession>,
    offsets: Query<&OffsetMarker>,
) {
    for key in keyboard_input.get_just_pressed() {
        if let KeyCode::Return = key {
            let mut offsets: Vec<&OffsetMarker> = offsets.iter().collect();
            offsets.sort_by_key(|marker| marker.num);
            let mut start_time = session.estimated_time_first_song;
            for marker in offsets.iter() {
                let song = &session.songs[marker.num];
                let end_time = start_time + song.length;
                dbg!(song, start_time, end_time);
                cut_song(
                    &session,
                    song,
                    start_time + marker.pos,
                    end_time + marker.pos,
                )
                .unwrap();
                start_time = start_time + song.length;
            }
        }
    }
}
