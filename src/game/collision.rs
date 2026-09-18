//! Collision detection and components.

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::{game::movement::MovementController, world::tiledhelper::Obstacle};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Collider>();
}

/// A 2D rectangular bounding-box collider.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Collider {
    /// Half-size of the collision box in world units (half width, half height).
    pub half_size: Vec2,
    /// Center offset from the entity's transform translation in world units.
    pub offset: Vec2,
}

#[allow(dead_code)]
impl Collider {
    /// Creates a centered collider with the specified half-size.
    pub fn new(half_size: Vec2) -> Self {
        Self {
            half_size,
            offset: Vec2::ZERO,
        }
    }

    /// Creates a centered collider from total width and height.
    pub fn from_size(size: Vec2) -> Self {
        Self {
            half_size: size * 0.5,
            offset: Vec2::ZERO,
        }
    }

    /// Sets an offset for the collider relative to the entity transform.
    pub fn with_offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }

    /// Returns the world-space bounding box `(min, max)` for this collider at position `pos`.
    pub fn aabb(&self, pos: Vec2) -> (Vec2, Vec2) {
        let center = pos + self.offset;
        (center - self.half_size, center + self.half_size)
    }

    /// Checks if this collider at `pos` intersects another collider at `other_pos`.
    pub fn intersects(&self, pos: Vec2, other: &Collider, other_pos: Vec2) -> bool {
        let (min_a, max_a) = self.aabb(pos);
        let (min_b, max_b) = other.aabb(other_pos);

        min_a.x < max_b.x && max_a.x > min_b.x && min_a.y < max_b.y && max_a.y > min_b.y
    }
}

impl Default for Collider {
    fn default() -> Self {
        Self {
            half_size: Vec2::new(16.0, 16.0),
            offset: Vec2::ZERO,
        }
    }
}

/// Checks whether an entity moving from `current_pos` to `new_pos` collides with any tilemap obstacles,
/// static colliders, or other moving entities with colliders.
pub fn check_collision(
    current_pos: Vec2,
    new_pos: Vec2,
    collider: &Collider,
    current_entity: Entity,
    other_moving_entities: &[(Entity, Vec2, Collider)],
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
    static_colliders: &Query<(Entity, &Transform, &Collider), Without<MovementController>>,
) -> bool {
    // 1. Check tilemap obstacles
    let (min, max) = collider.aabb(new_pos);
    if check_tilemap_collision(min, max, tilemap_q, obstacle_q) {
        return true;
    }

    // 2. Check static colliders (entities with Collider but without MovementController)
    for (other_entity, other_transform, other_collider) in static_colliders.iter() {
        if other_entity == current_entity {
            continue;
        }
        let other_pos = other_transform.translation.xy();
        if collider.intersects(new_pos, other_collider, other_pos) {
            let was_intersecting = collider.intersects(current_pos, other_collider, other_pos);
            if !was_intersecting {
                return true;
            }
            let prev_dist_sq = (current_pos - other_pos).length_squared();
            let new_dist_sq = (new_pos - other_pos).length_squared();
            if new_dist_sq < prev_dist_sq {
                return true;
            }
        }
    }

    // 3. Check moving colliders (other entities with MovementController)
    for (other_entity, other_pos, other_collider) in other_moving_entities {
        if *other_entity == current_entity {
            continue;
        }
        if collider.intersects(new_pos, other_collider, *other_pos) {
            let was_intersecting = collider.intersects(current_pos, other_collider, *other_pos);
            if !was_intersecting {
                return true;
            }
            let prev_dist_sq = (current_pos - *other_pos).length_squared();
            let new_dist_sq = (new_pos - *other_pos).length_squared();
            // If moving closer or penetrating deeper, block it;
            // if moving away to separate, allow it so entities don't get permanently stuck.
            if new_dist_sq < prev_dist_sq {
                return true;
            }
        }
    }

    false
}

/// Checks whether the bounding box from `min_world` to `max_world` intersects any obstacle tile.
pub fn check_tilemap_collision(
    min_world: Vec2,
    max_world: Vec2,
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
    for (map_size, grid_size, tile_size, map_type, tile_storage, map_transform, anchor) in
        tilemap_q.iter()
    {
        let map_inv = map_transform.to_matrix().inverse();

        // Transform bounding box corners to map coordinates
        let corners = [
            (map_inv * min_world.extend(0.0).extend(1.0)).xy(),
            (map_inv * Vec2::new(max_world.x, min_world.y).extend(0.0).extend(1.0)).xy(),
            (map_inv * max_world.extend(0.0).extend(1.0)).xy(),
            (map_inv * Vec2::new(min_world.x, max_world.y).extend(0.0).extend(1.0)).xy(),
        ];

        let min_map = Vec2::new(
            corners.iter().map(|c| c.x).fold(f32::INFINITY, f32::min),
            corners.iter().map(|c| c.y).fold(f32::INFINITY, f32::min),
        );
        let max_map = Vec2::new(
            corners.iter().map(|c| c.x).fold(f32::NEG_INFINITY, f32::max),
            corners.iter().map(|c| c.y).fold(f32::NEG_INFINITY, f32::max),
        );

        let offset = anchor.as_offset(map_size, grid_size, tile_size, map_type);
        let min_pos = min_map - offset;
        let max_pos = max_map - offset;

        // Inward epsilon to prevent edge-touching from triggering a collision when sliding along tiles
        const EPSILON: f32 = 0.5;
        let min_x = (((min_pos.x + EPSILON) / grid_size.x) + 0.5).floor() as i32;
        let max_x = (((max_pos.x - EPSILON) / grid_size.x) + 0.5).floor() as i32;
        let min_y = (((min_pos.y + EPSILON) / grid_size.y) + 0.5).floor() as i32;
        let max_y = (((max_pos.y - EPSILON) / grid_size.y) + 0.5).floor() as i32;

        let start_x = min_x.clamp(0, map_size.x as i32 - 1) as u32;
        let end_x = max_x.clamp(0, map_size.x as i32 - 1) as u32;
        let start_y = min_y.clamp(0, map_size.y as i32 - 1) as u32;
        let end_y = max_y.clamp(0, map_size.y as i32 - 1) as u32;

        if min_x <= end_x as i32
            && max_x >= start_x as i32
            && min_y <= end_y as i32
            && max_y >= start_y as i32
        {
            for tx in start_x..=end_x {
                for ty in start_y..=end_y {
                    let tile_pos = TilePos { x: tx, y: ty };
                    if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                        if obstacle_q.get(tile_entity).is_ok() {
                            return true;
                        }
                    }
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collider_aabb() {
        let collider = Collider::new(Vec2::new(10.0, 20.0));
        let (min, max) = collider.aabb(Vec2::new(50.0, 100.0));
        assert_eq!(min, Vec2::new(40.0, 80.0));
        assert_eq!(max, Vec2::new(60.0, 120.0));
    }

    #[test]
    fn test_collider_with_offset() {
        let collider = Collider::new(Vec2::new(10.0, 10.0)).with_offset(Vec2::new(5.0, -5.0));
        let (min, max) = collider.aabb(Vec2::ZERO);
        assert_eq!(min, Vec2::new(-5.0, -15.0));
        assert_eq!(max, Vec2::new(15.0, 5.0));
    }

    #[test]
    fn test_collider_intersects() {
        let col_a = Collider::new(Vec2::new(10.0, 10.0));
        let col_b = Collider::new(Vec2::new(10.0, 10.0));

        // Overlapping
        assert!(col_a.intersects(Vec2::ZERO, &col_b, Vec2::new(15.0, 0.0)));
        // Touching edge (not strictly overlapping)
        assert!(!col_a.intersects(Vec2::ZERO, &col_b, Vec2::new(20.0, 0.0)));
        // Separate
        assert!(!col_a.intersects(Vec2::ZERO, &col_b, Vec2::new(50.0, 50.0)));
    }
}
