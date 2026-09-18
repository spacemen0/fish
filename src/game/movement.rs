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

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::{
    AppSystems,
    game::collision::{Collider, check_collision},
    states::GameState,
    world::tiledhelper::Obstacle,
};

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
    mut movement_query: Query<(Entity, &MovementController, &mut Transform, Option<&Collider>)>,
    static_colliders: Query<(Entity, &Transform, &Collider), Without<MovementController>>,
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

    // Snapshot positions of moving colliders so entities can check against each other without query conflict
    let mut moving_colliders: Vec<(Entity, Vec2, Collider)> = movement_query
        .iter()
        .filter_map(|(entity, _, transform, collider)| {
            collider.map(|col| (entity, transform.translation.xy(), col.clone()))
        })
        .collect();

    for (entity, controller, mut transform, maybe_collider) in &mut movement_query {
        if controller.intent == Vec2::ZERO {
            continue;
        }

        let velocity = controller.max_speed * controller.intent;
        let delta_movement = velocity * delta_time;

        if let Some(collider) = maybe_collider {
            let current_pos = transform.translation.xy();

            // Try moving on X axis
            let mut new_pos = current_pos;
            new_pos.x += delta_movement.x;
            if !check_collision(
                current_pos,
                new_pos,
                collider,
                entity,
                &moving_colliders,
                &tilemap_q,
                &obstacle_q,
                &static_colliders,
            ) {
                transform.translation.x = new_pos.x;
            }

            // Try moving on Y axis (from the updated intermediate position)
            let intermediate_pos = transform.translation.xy();
            let mut final_pos = intermediate_pos;
            final_pos.y += delta_movement.y;
            if !check_collision(
                intermediate_pos,
                final_pos,
                collider,
                entity,
                &moving_colliders,
                &tilemap_q,
                &obstacle_q,
                &static_colliders,
            ) {
                transform.translation.y = final_pos.y;
            }

            // Update snapshot position for this entity so subsequent entities see the new position
            if let Some(entry) = moving_colliders.iter_mut().find(|(e, _, _)| *e == entity) {
                entry.1 = transform.translation.xy();
            }
        } else {
            transform.translation.x += delta_movement.x;
            transform.translation.y += delta_movement.y;
        }
    }
}
