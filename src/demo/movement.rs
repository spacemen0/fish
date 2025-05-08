//! Handle player input and translate it into movement through a character
//! controller. A character controller is the collection of systems that govern
//! the movement of characters.
//!
//! In our case, the character controller has the following logic:
//! - Set [`MovementController`] intent based on directional keyboard input.
//!   This is done in the `player` module, as it is specific to the player
//!   character.
//! - Apply movement based on [`MovementController`] intent and maximum speed.
//! - Wrap the character within the window.
//!
//! Note that the implementation used here is limited for demonstration
//! purposes. If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/blob/main/examples/movement/physics_in_fixed_timestep.rs).

use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::WindowResized;

use crate::AppSet;
use crate::demo::player::Player;
use crate::screens::Screen;
use crate::world::tilemap::{MAP_HEIGHT, MAP_WIDTH, TILE_SCALE, TILE_SIZE};

use super::player::{PLAYER_SCALE, PLAYER_SIZE};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<MovementController>();
    app.register_type::<WithinBoundrie>();
    app.init_resource::<CameraBounds>();
    app.add_event::<CameraScaleEvent>();
    app.add_systems(
        Update,
        (
            apply_movement,
            apply_screen_wrap,
            camera_follow_player,
            camera_zoom,
        )
            .chain()
            .run_if(in_state(Screen::Gameplay))
            .in_set(AppSet::Update),
    );
    app.add_systems(OnEnter(Screen::Gameplay), calculate_camera_bounds);
    app.add_systems(
        Update,
        calculate_camera_bounds
            .run_if(on_event::<WindowResized>.or(on_event::<CameraScaleEvent>))
            .in_set(AppSet::PreUpdate),
    );
    app.add_systems(
        Update,
        camera_zoom
            .run_if(on_event::<MouseWheel>.and(in_state(Screen::Gameplay)))
            .in_set(AppSet::PreUpdate),
    );
}

/// These are the movement parameters for our character controller.
/// For now, this is only used for a single player, but it could power NPCs or
/// other players as well.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MovementController {
    /// The direction the character wants to move in.
    pub intent: Vec2,

    /// Maximum speed in world units per second.
    /// 1 world unit = 1 pixel when using the default 2D camera and no physics engine.
    pub max_speed: f32,
}

impl Default for MovementController {
    fn default() -> Self {
        Self {
            intent: Vec2::ZERO,
            // 400 pixels per second is a nice default, but we can still vary this per character.
            max_speed: 400.0,
        }
    }
}

fn apply_movement(
    time: Res<Time>,
    mut movement_query: Query<(&MovementController, &mut Transform)>,
) {
    for (controller, mut transform) in &mut movement_query {
        let velocity = controller.max_speed * controller.intent;
        transform.translation += velocity.extend(0.0) * time.delta_secs();
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct WithinBoundrie;

fn apply_screen_wrap(mut wrap_query: Query<&mut Transform, With<WithinBoundrie>>) {
    let player_size = PLAYER_SIZE * PLAYER_SCALE;
    let width = MAP_WIDTH as f32 * TILE_SIZE as f32 * TILE_SCALE;
    let half_width = width / 2.0 - player_size / 2.0;
    let height = MAP_HEIGHT as f32 * TILE_SIZE as f32 * TILE_SCALE;
    let half_height = height / 2.0 - player_size / 2.0;
    for mut transform in &mut wrap_query {
        let position = transform.translation.xy();
        let clamped_x = position.x.clamp(-half_width + 4.0, half_width - 4.0);
        let clamped_y = position.y.clamp(-half_height + 8.0, half_height - 4.0);
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
    target_y = target_y.clamp(camera_bounds.min.y, camera_bounds.max.y);

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
        ortho.scale = ortho.scale.clamp(0.8, 1.0);
        ew.write(CameraScaleEvent);
    }
}
