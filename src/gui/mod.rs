use bevy::prelude::*;
use bevy_prototype_lyon::{
    entity::ShapeBundle,
    plugin::ShapePlugin,
    prelude::{DrawMode, FillOptions, GeometryBuilder, PathBuilder, ShapeColors, StrokeOptions},
};


use crate::{cut::{get_named_excerpts, NamedExcerpt}, recording_session::RecordingSession};

static Y_OFFSET_PER_SONG: f32 = -120.0;
static SONG_WIDTH: f32 = 800.0;
static SONG_HEIGHT: f32 = 100.0;

pub fn get_offsets(session: RecordingSession) -> Vec<f64> {
    App::build()
        .add_plugins(DefaultPlugins)
        .insert_resource(session)
        // .insert_resource(Msaa { samples: 8 })
        .add_plugin(ShapePlugin)
        .add_startup_system(initialize_camera_system.system())
        .add_startup_system(add_excerpts_system.system())
        .add_system(show_excerpts_system.system())
        .run();
    todo!()
}

struct ExcerptNum(usize);

fn add_excerpts_system(mut commands: Commands, session: Res<RecordingSession>) {
    let excerpts = get_named_excerpts(&session);
    for (i, excerpt) in excerpts.into_iter().enumerate() {
        commands.spawn().insert(excerpt).insert(ExcerptNum(i));
    }
}

fn show_excerpts_system(
    mut commands: Commands,
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
    }
}

fn get_path_for_excerpt(excerpt: &NamedExcerpt, num: &ExcerptNum) -> PathBuilder {
    let mut path = PathBuilder::new();
    let values = [0., 0.5, 1.0, 0.7, 0.3, 0.9];
    let y_offset = (num.0 as f32) * Y_OFFSET_PER_SONG;
    path.line_to(Vec2::new(0.0, y_offset));
    for (i, y) in values.iter().enumerate() {
        let x = (i as f32) / (values.len() as f32);
        path.line_to(Vec2::new(x * SONG_WIDTH, y_offset + *y * SONG_HEIGHT));
    }
    path
}

fn initialize_camera_system(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}
