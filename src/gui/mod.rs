mod config;
mod text;

use self::{
    config::{
        LEFT_SONG_TEXT_X_OFFSET, RIGHT_SONG_TEXT_X_OFFSET, SONG_HEIGHT, SONG_WIDTH,
        Y_OFFSET_PER_SONG,
    },
    text::get_text_bundle_for_song,
};
use crate::{
    audio_excerpt::AudioExcerpt,
    config::NUM_OFFSETS_TO_TRY,
    cut::{get_named_excerpts, NamedExcerpt},
    recording_session::RecordingSession,
};
use bevy::prelude::*;
use bevy_prototype_lyon::{
    plugin::ShapePlugin,
    prelude::{DrawMode, FillOptions, GeometryBuilder, PathBuilder, ShapeColors, StrokeOptions},
};

pub fn get_offsets(session: RecordingSession) -> Vec<f64> {
    App::build()
        .add_plugins(DefaultPlugins)
        .insert_resource(session)
        // .insert_resource(Msaa { samples: 8 })
        .add_plugin(ShapePlugin)
        .add_startup_system(initialize_camera_system.system())
        .add_startup_system(add_excerpts_system.system())
        .add_system(show_excerpts_system.system())
        .add_system(centering_system.system())
        .run();
    todo!()
}

struct ExcerptNum(usize);

struct OffsetMarker(f32);

struct TextPosition {
    x: f32,
    y: f32,
}

fn add_excerpts_system(mut commands: Commands, session: Res<RecordingSession>) {
    let excerpts = get_named_excerpts(&session);
    for (i, excerpt) in excerpts.into_iter().enumerate() {
        commands
            .spawn()
            .insert(excerpt)
            .insert(ExcerptNum(i))
            .insert(OffsetMarker(0.0));
    }
}

fn show_excerpts_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    excerpts: Query<(Entity, &NamedExcerpt, &ExcerptNum), Without<Draw>>,
) {
    let invisible = Color::Rgba {
        red: 0.0,
        green: 0.0,
        blue: 0.0,
        alpha: 0.0,
    };
    for (entity, excerpt, num) in excerpts.iter() {
        let path = get_path_for_excerpt(excerpt, num);
        commands
            .entity(entity)
            .insert_bundle(GeometryBuilder::build_as(
                &path.build(),
                ShapeColors::outlined(invisible, Color::BLACK),
                DrawMode::Outlined {
                    fill_options: FillOptions::default(),
                    outline_options: StrokeOptions::default().with_line_width(10.0),
                },
                Transform::default(),
            ));
        let get_y_position = |song_num| song_num as f32 * Y_OFFSET_PER_SONG;
        commands
            .spawn()
            .insert_bundle(get_text_bundle_for_song(
                &asset_server,
                &excerpt.song,
                HorizontalAlign::Center,
            ))
            .insert(TextPosition {
                x: LEFT_SONG_TEXT_X_OFFSET,
                y: get_y_position(num.0 + 1),
            });
        commands
            .spawn()
            .insert_bundle(get_text_bundle_for_song(
                &asset_server,
                &excerpt.song,
                HorizontalAlign::Center,
            ))
            .insert(TextPosition {
                x: RIGHT_SONG_TEXT_X_OFFSET,
                y: get_y_position(num.0),
            });
    }
}

fn get_path_for_excerpt(excerpt: &NamedExcerpt, num: &ExcerptNum) -> PathBuilder {
    let mut path = PathBuilder::new();
    let values = get_volume_data(&excerpt.excerpt);
    let y_offset = (num.0 as f32) * Y_OFFSET_PER_SONG;
    path.line_to(Vec2::new(0.0, y_offset));
    for (i, y) in values.iter().enumerate() {
        let x = (i as f32) / (values.len() as f32);
        path.line_to(Vec2::new(
            x * SONG_WIDTH,
            y_offset + (*y as f32) * SONG_HEIGHT,
        ));
    }
    path
}

fn get_volume_data(excerpt: &AudioExcerpt) -> Vec<f64> {
    let width = excerpt.end.time - excerpt.start.time;
    let step_size = width / NUM_OFFSETS_TO_TRY as f64;
    let times = (1..NUM_OFFSETS_TO_TRY).map(|x| excerpt.start.time + (x as f64) * step_size);
    times.map(|time| excerpt.get_volume_at(time)).collect()
}

fn initialize_camera_system(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn centering_system(mut query: Query<(&mut Transform, &TextPosition), With<Text>>) {
    for (mut transform, pos) in query.iter_mut() {
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
        dbg!(transform.translation);
    }
}
