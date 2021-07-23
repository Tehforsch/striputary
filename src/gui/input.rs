use bevy::{app::AppExit, input::mouse::MouseWheel, prelude::*, render::camera::Camera};

use super::{
    config::{SONG_HEIGHT, SONG_Y_START, Y_OFFSET_PER_SONG},
    OffsetMarker, ScrollPosition,
};

#[derive(Default, Debug)]
pub struct MousePosition(Vec2);

fn check_inside_excerpt(world_pos: Vec2, excerpt_num: usize) -> bool {
    let y_pos =
        (world_pos.y - SONG_Y_START - Y_OFFSET_PER_SONG * (excerpt_num as f32)) / SONG_HEIGHT;
    return y_pos >= 0.0 && y_pos <= 1.0;
}

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
    }
}

pub fn move_markers_on_click_system(
    mut markers: Query<&mut OffsetMarker>,
    mouse_button_input: Res<Input<MouseButton>>,
    mouse_pos: Res<MousePosition>,
) {
    for event in mouse_button_input.get_pressed() {
        if let MouseButton::Left = event {
            let mut sorted_markers: Vec<Mut<OffsetMarker>> = markers.iter_mut().collect();
            let mut clicked = false;
            sorted_markers.sort_by_key(|marker| marker.num);
            for mut marker in sorted_markers.into_iter() {
                if clicked || check_inside_excerpt(mouse_pos.0, marker.num) {
                    marker.pos = mouse_pos.0.x as f64;
                    clicked = true;
                }
            }
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
