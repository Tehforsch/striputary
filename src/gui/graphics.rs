use bevy::prelude::*;
use bevy_prototype_lyon::{
    entity::ShapeBundle,
    prelude::{DrawMode, FillOptions, GeometryBuilder, PathBuilder, ShapeColors, StrokeOptions},
};

use crate::{cut::NamedExcerpt, song::Song};

use super::{
    config::{
        LINE_WIDTH, MARKER_HEIGHT, SONG_HEIGHT, SONG_TEXT_X_DISTANCE, SONG_X_END, SONG_X_START,
        SONG_Y_START, Y_OFFSET_PER_SONG,
    },
    get_volume_data, ExcerptNum, OffsetMarker,
};

pub struct TextPosition {
    pub x: f32,
    pub y: f32,
}

pub fn show_excerpts_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    excerpts: Query<(Entity, &NamedExcerpt, &ExcerptNum), Without<Draw>>,
) {
    for (entity, excerpt, num) in excerpts.iter() {
        spawn_path_for_excerpt(&mut commands, excerpt, num, entity);
        let get_y_position = |song_num| song_num as f32 * Y_OFFSET_PER_SONG;
        spawn_text_for_excerpt(
            &mut commands,
            &asset_server,
            &excerpt.song,
            TextPosition {
                x: SONG_X_START - SONG_TEXT_X_DISTANCE,
                y: get_y_position(num.0 + 1),
            },
        );
        spawn_text_for_excerpt(
            &mut commands,
            &asset_server,
            &excerpt.song,
            TextPosition {
                x: SONG_X_END + SONG_TEXT_X_DISTANCE,
                y: get_y_position(num.0),
            },
        );
    }
}

fn spawn_text_for_excerpt(
    commands: &mut Commands,
    asset_server: &AssetServer,
    song: &Song,
    text_position: TextPosition,
) {
    commands
        .spawn()
        .insert_bundle(get_text_bundle_for_song(&asset_server, &song))
        .insert(text_position);
}

fn get_text_bundle_for_song(asset_server: &AssetServer, song: &Song) -> Text2dBundle {
    Text2dBundle {
        text: Text::with_section(
            &format!("{}\n{}", song.artist, song.title),
            TextStyle {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 30.0,
                color: Color::BLACK,
            },
            TextAlignment {
                horizontal: HorizontalAlign::Center,
                ..Default::default()
            },
        ),
        ..Default::default()
    }
}

fn spawn_path_for_excerpt(
    commands: &mut Commands,
    excerpt: &NamedExcerpt,
    num: &ExcerptNum,
    entity: Entity,
) {
    let volume_data = get_volume_data(&excerpt.excerpt);
    let path = get_path_for_excerpt(volume_data, num);
    commands
        .entity(entity)
        .insert_bundle(get_shape_bundle_for_path(path, LINE_WIDTH, Color::BLACK));
}

fn get_shape_bundle_for_path(path: PathBuilder, line_width: f32, color: Color) -> ShapeBundle {
    let invisible = Color::Rgba {
        red: 0.0,
        green: 0.0,
        blue: 0.0,
        alpha: 0.0,
    };
    GeometryBuilder::build_as(
        &path.build(),
        ShapeColors::outlined(invisible, color),
        DrawMode::Outlined {
            fill_options: FillOptions::default(),
            outline_options: StrokeOptions::default().with_line_width(line_width),
        },
        Transform::default(),
    )
}

fn get_path_for_excerpt(volume_data: Vec<f64>, num: &ExcerptNum) -> PathBuilder {
    let mut path = PathBuilder::new();
    let y_offset = (num.0 as f32) * Y_OFFSET_PER_SONG;
    let width = SONG_X_END - SONG_X_START;
    path.move_to(Vec2::new(SONG_X_START, SONG_Y_START + y_offset));
    for (i, y) in volume_data.iter().enumerate() {
        let x = (i as f32) / (volume_data.len() as f32);
        path.line_to(Vec2::new(
            SONG_X_START + x * width,
            SONG_Y_START + y_offset + (*y as f32) * SONG_HEIGHT,
        ));
    }
    path
}

pub fn spawn_offset_markers_system(
    mut commands: Commands,
    query: Query<(Entity, &OffsetMarker, &ExcerptNum), Without<Draw>>,
) {
    for (entity, _, num) in query.iter() {
        let mut path = PathBuilder::new();
        let middle = (SONG_X_START + SONG_X_END) * 0.5;
        let y_offset = SONG_Y_START + (num.0 as f32) * Y_OFFSET_PER_SONG;
        path.move_to(Vec2::new(middle, y_offset));
        path.line_to(Vec2::new(middle, y_offset + MARKER_HEIGHT));
        commands
            .entity(entity)
            .insert_bundle(get_shape_bundle_for_path(path, 2.0, Color::RED));
    }
}
