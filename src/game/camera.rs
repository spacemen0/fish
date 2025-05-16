use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    window::{PrimaryWindow, WindowResized},
};

use crate::{AppSystems, constants::*, screens::Screen};

use super::player::Player;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<WithinBounds>();
    app.init_resource::<CameraBounds>();
    app.init_resource::<CursorPos>();
    app.add_event::<CameraScaleEvent>();
    app.add_systems(OnEnter(Screen::Gameplay), calculate_camera_bounds);

    app.add_systems(
        Update,
        (
            camera_zoom.run_if(on_event::<MouseWheel>),
            update_cursor_pos,
            apply_screen_wrap,
            camera_follow_player,
            calculate_camera_bounds
                .run_if(on_event::<WindowResized>.or(on_event::<CameraScaleEvent>)),
        )
            .in_set(AppSystems::PostUpdate)
            .run_if(in_state(Screen::Gameplay)),
    );
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct WithinBounds;

const WRAP_Y_OFFSET: f32 = 12.0;

fn apply_screen_wrap(mut wrap_query: Query<&mut Transform, With<WithinBounds>>) {
    let player_size_x = (GRID_SIZE_X - 8) as f32 * PLAYER_SCALE;
    let player_size_y = (GRID_SIZE_Y - 14) as f32 * PLAYER_SCALE;
    let width = MAP_WIDTH as f32 * TILE_SIZE as f32 * TILE_SCALE;
    let half_width = width / 2.0 - player_size_x / 2.0;
    let height = MAP_HEIGHT as f32 * TILE_SIZE as f32 * TILE_SCALE;
    let half_height = height / 2.0 - player_size_y / 2.0;
    for mut transform in &mut wrap_query {
        let position = transform.translation.xy();
        let clamped_x = position.x.clamp(-half_width, half_width);
        let clamped_y = position.y.clamp(-half_height + WRAP_Y_OFFSET, half_height);
        transform.translation = Vec3::new(clamped_x, clamped_y, transform.translation.z);
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct CameraBounds {
    pub min: Vec2,
    pub max: Vec2,
}

#[derive(Event)]
pub struct CameraScaleEvent;

impl FromWorld for CameraBounds {
    fn from_world(_world: &mut World) -> Self {
        Self {
            min: vec2(0.0, 0.0),
            max: vec2(0.0, 0.0),
        }
    }
}

fn calculate_camera_bounds(
    mut camera_bounds: ResMut<CameraBounds>,
    windows: Query<&Window>,
    projection_query: Query<&Projection, With<Camera2d>>,
) {
    let window = windows.single().expect("Window should exist!");
    let projection = projection_query
        .single()
        .expect("Camera projection should exist!");

    let (win_w, win_h, scale) = match projection {
        Projection::Orthographic(ortho) => (window.width(), window.height(), ortho.scale),
        _ => (window.width(), window.height(), 1.0),
    };

    let half_visible_w = (win_w * 0.5) * scale;
    let half_visible_h = (win_h * 0.5) * scale;

    let map_width = MAP_WIDTH as f32 * TILE_SIZE as f32 * TILE_SCALE;
    let map_height = MAP_HEIGHT as f32 * TILE_SIZE as f32 * TILE_SCALE;
    let half_map_w = map_width / 2.0;
    let half_map_h = map_height / 2.0;

    camera_bounds.min.x = -half_map_w + half_visible_w;
    camera_bounds.max.x = half_map_w - half_visible_w;
    camera_bounds.min.y = -half_map_h + half_visible_h;
    camera_bounds.max.y = half_map_h - half_visible_h;
}

fn camera_follow_player(
    _time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    camera_bounds: Res<CameraBounds>,
) {
    let player_transform = player_query.single().expect("Player should exist!");
    let mut camera_transform = camera_query.single_mut().expect("Camera should exist!");

    let player_pos = player_transform.translation.xy();
    let mut target_x = player_pos.x;
    let mut target_y = player_pos.y;
    target_x = target_x.clamp(camera_bounds.min.x, camera_bounds.max.x);
    target_y = target_y.clamp(camera_bounds.min.y - WRAP_Y_OFFSET, camera_bounds.max.y);

    let target_position = Vec3::new(target_x, target_y, camera_transform.translation.z);

    // let smoothness: f32 = 0.75;
    // let t = 1.0 - smoothness.powf(time.delta_secs() * 10.0);

    camera_transform.translation = target_position;
}

fn camera_zoom(
    mut scroll_evr: EventReader<MouseWheel>,
    mut ew: EventWriter<CameraScaleEvent>,
    mut query: Query<&mut Projection, With<Camera2d>>,
) {
    // Calculate the total scroll amount from all events
    let scroll_amount = scroll_evr.read().fold(0.0, |acc, ev| {
        acc + match ev.unit {
            MouseScrollUnit::Line => ev.y,
            MouseScrollUnit::Pixel => ev.y / 100.0,
        }
    });

    if scroll_amount == 0.0 {
        return;
    }

    // Apply zoom to all 2D cameras
    let mut projection = query.single_mut().expect("Camera should exist!");
    // Adjust zoom speed/sensitivity
    let zoom_speed = 0.1;

    // Adjust scale - smaller values zoom in
    if let Projection::Orthographic(ref mut ortho) = *projection {
        ortho.scale *= 1.0 - scroll_amount * zoom_speed;
        // Clamp to reasonable limits
        ortho.scale = ortho.scale.clamp(0.2, 1.0);
        ew.write(CameraScaleEvent);
    }
}
#[derive(Resource)]
pub struct CursorPos(pub Vec2);
impl Default for CursorPos {
    fn default() -> Self {
        // Initialize the cursor pos at some far away place. It will get updated
        // correctly when the cursor moves.
        Self(Vec2::new(-1000.0, -1000.0))
    }
}
fn update_cursor_pos(
    camera_q: Query<(&GlobalTransform, &Camera)>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut cursor_pos: ResMut<CursorPos>,
) {
    let (cam_t, cam) = camera_q.single().expect("Camera should exist!");
    let window = q_window.single().expect("Window should exist!");
    if let Some(pos) = window.cursor_position() {
        // Convert the cursor position to world space
        if let Ok(pos) = cam.viewport_to_world_2d(cam_t, pos) {
            *cursor_pos = CursorPos(pos);
        }
    }
}
