use bevy::{prelude::*, render::camera::Camera};
use bevy_prototype_lyon::{
    entity::ShapeBundle,
    prelude::{DrawMode, FillOptions, GeometryBuilder, PathBuilder, ShapeColors, StrokeOptions},
};

use crate::{excerpt_collection::NamedExcerpt, excerpt_collections::ExcerptCollections, song::Song};

use super::{config::{
        LINE_WIDTH, MARKER_HEIGHT, SONG_HEIGHT, SONG_TEXT_X_DISTANCE, SONG_TEXT_Y_OFFSET,
        SONG_X_END, SONG_X_START, SONG_Y_START, Y_DISTANCE_PER_MOUSEWHEEL_TICK, Y_OFFSET_PER_SONG,
    }, excerpt_view::ExcerptView, offset_marker::PositionMarker};

pub struct TextPosition {
    pub x: f32,
    pub y: f32,
}

pub enum ZLayer {
    Above,
}

pub struct ScrollPosition(pub i32);

pub fn show_excerpts_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    collections: Res<ExcerptCollections>,
    excerpt_views: Query<(Entity, &ExcerptView), Without<Draw>>,
) {
    let collection = collections.get_selected();
    for (entity, excerpt_view) in excerpt_views.iter() {
        let excerpt = collection.iter_excerpts().enumerate().find(|(i, _)| *i == excerpt_view.0).map(|(_, ex)| ex).unwrap();
        spawn_path_for_excerpt(&mut commands, excerpt, entity);
        let get_y_position = |song_num| song_num as f32 * Y_OFFSET_PER_SONG;
        spawn_text_for_excerpt(
            &mut commands,
            entity,
            &asset_server,
            &excerpt.song,
            TextPosition {
                x: SONG_X_START - SONG_TEXT_X_DISTANCE,
                y: get_y_position(excerpt.num + 1),
            },
        );
        spawn_text_for_excerpt(
            &mut commands,
            entity,
            &asset_server,
            &excerpt.song,
            TextPosition {
                x: SONG_X_END + SONG_TEXT_X_DISTANCE,
                y: get_y_position(excerpt.num),
            },
        );
    }
}

fn spawn_text_for_excerpt(
    commands: &mut Commands,
    parent_entity: Entity,
    asset_server: &AssetServer,
    song: &Song,
    text_position: TextPosition,
) {
    let text = commands.spawn()
        .insert_bundle(get_text_bundle_for_song(&asset_server, &song))
        .insert(text_position).id();
    commands.entity(parent_entity).push_children(&[text]);
}

fn get_text_bundle_for_song(asset_server: &AssetServer, song: &Song) -> Text2dBundle {
    Text2dBundle {
        text: Text::with_section(
            &format!("{}\n{}", song.artist, song.title),
            TextStyle {
                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
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

fn spawn_path_for_excerpt(commands: &mut Commands, excerpt: &NamedExcerpt, entity: Entity) {
    let path = get_path_for_excerpt(excerpt);
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

fn get_path_for_excerpt(excerpt: &NamedExcerpt) -> PathBuilder {
    let volume_data = excerpt.excerpt.get_volume_plot_data();
    let mut path = PathBuilder::new();
    let y_offset = (excerpt.num as f32) * Y_OFFSET_PER_SONG;
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
    query: Query<(Entity, &PositionMarker), Without<Draw>>,
) {
    for (entity, marker) in query.iter() {
        let mut path = PathBuilder::new();
        let middle = (SONG_X_START + SONG_X_END) * 0.5;
        let y_offset = SONG_Y_START + (marker.num as f32) * Y_OFFSET_PER_SONG;
        path.move_to(Vec2::new(middle, y_offset));
        path.line_to(Vec2::new(middle, y_offset + MARKER_HEIGHT));
        commands
            .entity(entity)
            .insert_bundle(get_shape_bundle_for_path(path, 2.0, Color::RED))
            .insert(ZLayer::Above);
    }
}

pub fn z_layering_system(mut query: Query<(&mut Transform, &ZLayer)>) {
    for (mut transform, z_layer) in query.iter_mut() {
        transform.translation.z = match z_layer {
            ZLayer::Above => 1.0,
        }
    }
}

pub fn text_positioning_system(mut query: Query<(&mut Transform, &TextPosition), With<Text>>) {
    for (mut transform, pos) in query.iter_mut() {
        transform.translation.x = pos.x;
        transform.translation.y = pos.y + SONG_TEXT_Y_OFFSET;
    }
}

pub fn marker_positioning_system(mut query: Query<(&mut Transform, &PositionMarker)>) {
    for (mut transform, marker) in query.iter_mut() {
        transform.translation.x = marker.get_world_pos();
    }
}

pub fn camera_positioning_system(
    mut camera: Query<&mut Transform, With<Camera>>,
    windows: Res<Windows>,
    scroll_position: Res<ScrollPosition>,
) {
    let window = windows.get_primary().unwrap();
    camera.single_mut().unwrap().translation.x = 0.0;
    camera.single_mut().unwrap().translation.y =
        -window.height() / 2.0 + scroll_position.0 as f32 * Y_DISTANCE_PER_MOUSEWHEEL_TICK;
}

pub fn initialize_camera_system(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}
