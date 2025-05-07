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

use crate::AppSet;
use crate::demo::player::Player;
use crate::screens::Screen;
use crate::world::tilemap::{MAP_HEIGHT, MAP_WIDTH, TILE_SCALE, TILE_SIZE};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<MovementController>();
    app.register_type::<WithinBoundrie>();

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
    let sprite_size = 64.0; // Player sprite size
    let width = MAP_WIDTH as f32 * TILE_SIZE as f32 * TILE_SCALE;
    let half_width = width / 2.0 - sprite_size / 2.0;
    let height = MAP_HEIGHT as f32 * TILE_SIZE as f32 * TILE_SCALE;
    let half_height = height / 2.0 - sprite_size / 2.0;
    for mut transform in &mut wrap_query {
        let position = transform.translation.xy();
        let clamped_x = position.x.clamp(-half_width, half_width);
        let clamped_y = position.y.clamp(-half_height, half_height);
        transform.translation = Vec3::new(clamped_x, clamped_y, transform.translation.z);
    }
}

fn camera_follow_player(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut camera_translation: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    let player_transform = player_query.single().unwrap();

    let mut camera_transform = camera_translation.single_mut().unwrap();
    let player_pos = player_transform.translation.xy();
    let target_position = player_pos.extend(camera_transform.translation.z);

    let smoothness: f32 = 0.75;

    let t = 1.0 - smoothness.powf(time.delta_secs() * 10.0);

    camera_transform.translation = camera_transform.translation.lerp(target_position, t);
}

fn camera_zoom(
    mut scroll_evr: EventReader<MouseWheel>,
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
    let mut projection = query.single_mut().unwrap();
    // Adjust zoom speed/sensitivity
    let zoom_speed = 0.1;

    // Adjust scale - smaller values zoom in
    if let Projection::Orthographic(ref mut ortho) = *projection {
        ortho.scale *= 1.0 - scroll_amount * zoom_speed;
        // Clamp to reasonable limits
        ortho.scale = ortho.scale.clamp(0.1, 3.0);
    }
}
