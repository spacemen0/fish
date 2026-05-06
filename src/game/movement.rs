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

use bevy::{camera::primitives::Aabb, prelude::*};
use bevy_ecs_tilemap::prelude::*;

use crate::{AppSystems, states::GameState, world::tiledhelper::Obstacle};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<MovementController>();
    app.add_systems(
        Update,
        (apply_movement,)
            .run_if(in_state(GameState::Gameplay))
            .in_set(AppSystems::Update),
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
    mut movement_query: Query<(&MovementController, &mut Transform, &Aabb)>,
    tilemap_q: Query<
        (
            &TilemapSize,
            &TilemapGridSize,
            &TilemapTileSize,
            &TilemapType,
            &TileStorage,
            &Transform,
            &TilemapAnchor,
        ),
        Without<MovementController>,
    >,
    obstacle_q: Query<&Obstacle>,
) {
    let delta_time = time.delta_secs();

    for (controller, mut transform, aabb) in &mut movement_query {
        if controller.intent == Vec2::ZERO {
            continue;
        }

        let velocity = controller.max_speed * controller.intent;
        let delta_movement = velocity * delta_time;

        // Try moving on X axis
        let mut new_pos = transform.translation;
        new_pos.x += delta_movement.x;
        if !check_collision(new_pos, aabb, &tilemap_q, &obstacle_q) {
            transform.translation.x = new_pos.x;
        }

        // Try moving on Y axis
        let mut new_pos = transform.translation;
        new_pos.y += delta_movement.y;
        if !check_collision(new_pos, aabb, &tilemap_q, &obstacle_q) {
            transform.translation.y = new_pos.y;
        }
    }
}

fn check_collision(
    pos: Vec3,
    aabb: &Aabb,
    tilemap_q: &Query<
        (
            &TilemapSize,
            &TilemapGridSize,
            &TilemapTileSize,
            &TilemapType,
            &TileStorage,
            &Transform,
            &TilemapAnchor,
        ),
        Without<MovementController>,
    >,
    obstacle_q: &Query<&Obstacle>,
) -> bool {
    let half_extents = aabb.half_extents.xy();
    // Check 4 corners of the AABB
    let corners = [
        pos.xy() + Vec2::new(half_extents.x, half_extents.y),
        pos.xy() + Vec2::new(-half_extents.x, half_extents.y),
        pos.xy() + Vec2::new(half_extents.x, -half_extents.y),
        pos.xy() + Vec2::new(-half_extents.x, -half_extents.y),
    ];

    for (map_size, grid_size, tile_size, map_type, tile_storage, map_transform, anchor) in
        tilemap_q.iter()
    {
        let map_inv = map_transform.to_matrix().inverse();

        for corner in corners {
            let corner_in_map_pos = (map_inv * corner.extend(0.0).extend(1.0)).xy();

            if let Some(tile_pos) = TilePos::from_world_pos(
                &corner_in_map_pos,
                map_size,
                grid_size,
                tile_size,
                map_type,
                anchor,
            ) {
                if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                    if obstacle_q.get(tile_entity).is_ok() {
                        return true;
                    }
                }
            }
        }
    }

    false
}
