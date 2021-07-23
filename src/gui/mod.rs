mod config;
mod graphics;
mod mouse;

use self::{
    config::{SONG_TEXT_Y_OFFSET, Y_DISTANCE_PER_MOUSEWHEEL_TICK},
    graphics::{show_excerpts_system, spawn_offset_markers_system, TextPosition},
    mouse::{track_mouse_position_system, MousePosition},
};
use crate::{
    audio_excerpt::AudioExcerpt,
    config::NUM_OFFSETS_TO_TRY,
    cut::{cut_song, get_named_excerpts},
    recording_session::RecordingSession,
};
use bevy::{app::AppExit, input::mouse::MouseWheel, prelude::*, render::camera::Camera};
use bevy_prototype_lyon::plugin::ShapePlugin;

pub struct ExcerptNum(usize);

pub struct OffsetMarker(f64);

pub struct ScrollPosition(i32);

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
        .add_system(scrolling_input_system.system())
        .add_system(spawn_offset_markers_system.system())
        .add_system(exit_system.system())
        .add_system(cut_system.system())
        .add_system(track_mouse_position_system.system())
        .run();
}

fn add_excerpts_system(mut commands: Commands, session: Res<RecordingSession>) {
    let excerpts = get_named_excerpts(&session);
    for (i, excerpt) in excerpts.into_iter().enumerate() {
        commands.spawn().insert(excerpt).insert(ExcerptNum(i));
        commands
            .spawn()
            .insert(ExcerptNum(i))
            .insert(OffsetMarker(0.0));
    }
}

fn get_volume_data(excerpt: &AudioExcerpt) -> Vec<f64> {
    let width = excerpt.end.time - excerpt.start.time;
    let step_size = width / NUM_OFFSETS_TO_TRY as f64;
    let times = (1..NUM_OFFSETS_TO_TRY).map(|x| excerpt.start.time + (x as f64) * step_size);
    times.map(|time| excerpt.get_volume_at(time)).collect()
}

fn scrolling_input_system(
    mut mouse_wheel: EventReader<MouseWheel>,
    mut pos: ResMut<ScrollPosition>,
) {
    for event in mouse_wheel.iter() {
        if event.y < 0.0 {
            pos.0 -= 1;
        }
        if event.y > 0.0 {
            pos.0 += 1;
        }
    }
}

fn text_positioning_system(mut query: Query<(&mut Transform, &TextPosition), With<Text>>) {
    for (mut transform, pos) in query.iter_mut() {
        transform.translation.x = pos.x;
        transform.translation.y = pos.y + SONG_TEXT_Y_OFFSET;
    }
}

fn camera_positioning_system(
    mut camera: Query<&mut Transform, With<Camera>>,
    windows: Res<Windows>,
    scroll_position: Res<ScrollPosition>,
) {
    let window = windows.get_primary().unwrap();
    camera.single_mut().unwrap().translation.x = 0.0;
    camera.single_mut().unwrap().translation.y =
        -window.height() / 2.0 + scroll_position.0 as f32 * Y_DISTANCE_PER_MOUSEWHEEL_TICK;
}

fn initialize_camera_system(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn cut_system(
    keyboard_input: Res<Input<KeyCode>>,
    session: Res<RecordingSession>,
    offsets: Query<(&OffsetMarker, &ExcerptNum)>,
) {
    for key in keyboard_input.get_just_pressed() {
        if let KeyCode::Return = key {
            let mut offsets: Vec<(&OffsetMarker, &ExcerptNum)> = offsets.iter().collect();
            offsets.sort_by_key(|(_, num)| num.0);
            let mut start_time = session.estimated_time_first_song;
            for (marker, num) in offsets.iter() {
                let song = &session.songs[num.0];
                let end_time = start_time + song.length;
                dbg!(song, start_time, end_time);
                cut_song(&session, song, start_time + marker.0, end_time + marker.0).unwrap();
                start_time = start_time + song.length;
            }
        }
    }
}

fn exit_system(keyboard_input: Res<Input<KeyCode>>, mut app_exit_events: EventWriter<AppExit>) {
    for key in keyboard_input.get_just_pressed() {
        match key {
            KeyCode::Escape | KeyCode::Q => {
                app_exit_events.send(AppExit);
            }
            _ => {}
        }
    }
}
