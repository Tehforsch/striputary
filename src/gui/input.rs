use bevy::{app::AppExit, input::mouse::MouseWheel, prelude::*, render::camera::Camera};

use super::ScrollPosition;

pub fn scrolling_input_system(
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

pub fn exit_system(keyboard_input: Res<Input<KeyCode>>, mut app_exit_events: EventWriter<AppExit>) {
    for key in keyboard_input.get_just_pressed() {
        match key {
            KeyCode::Escape | KeyCode::Q => {
                app_exit_events.send(AppExit);
            }
            _ => {}
        }
    }
}

#[derive(Default)]
pub struct MousePosition(Vec2);

pub fn track_mouse_position_system(
    mut events_reader_cursor: EventReader<CursorMoved>,
    mut position: ResMut<MousePosition>,
    windows: Res<Windows>,
    camera_query: Query<&Transform, With<Camera>>,
) {
    let camera_transform = camera_query.single().unwrap();
    let window = windows.get_primary().unwrap();
    if let Some(cursor_pos_window) = events_reader_cursor.iter().next() {
        let size = Vec2::new(window.width() as f32, window.height() as f32);
        let p = cursor_pos_window.position - size / 2.0;
        let world_pos = camera_transform.compute_matrix() * p.extend(0.0).extend(1.0);
        position.0.x = world_pos.x;
        position.0.y = world_pos.y;
        dbg!(position.0);
    }
}
