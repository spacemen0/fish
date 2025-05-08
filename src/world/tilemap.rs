use bevy::prelude::*;

use crate::{AppSet, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    // app.register_type::<TileMap>();
    // app.register_type::<Tile>();
    // app.register_type::<TilePosition>();
    // app.register_type::<TileType>();
    // app.load_resource::<TileAssets>();

    app.add_systems(
        OnEnter(Screen::Gameplay),
        spawn_tile_map.in_set(AppSet::PreUpdate),
    );
}

// CONSTANTS
pub const TILE_SIZE: i32 = 16;
pub const TILE_SCALE: f32 = 3.0; // Scale for rendering
pub const MAP_WIDTH: i32 = 30; // For a larger farm
pub const MAP_HEIGHT: i32 = 20;

// /// Manages the entire map consisting of tiles
// #[derive(Component, Reflect, Default)]
// #[reflect(Component)]
// pub struct TileMap {
//     pub tiles: HashMap<(i32, i32), Entity>,
//     pub width: i32,
//     pub height: i32,
//     pub tile_size: i32,
// }

// #[derive(Resource, Asset, Clone, Reflect)]
// #[reflect(Resource)]
// pub struct TileAssets {
//     #[dependency]
//     asset: Handle<Image>,
//     texture_atlas: Handle<TextureAtlasLayout>,
// }

// impl FromWorld for TileAssets {
//     fn from_world(world: &mut World) -> Self {
//         let assets = world.resource::<AssetServer>();
//         Self {
//             asset: assets.load_with_settings(
//                 "images/tiles.png",
//                 |settings: &mut ImageLoaderSettings| {
//                     // Use `nearest` image sampling to preserve pixel art style.
//                     settings.sampler = ImageSampler::nearest();
//                 },
//             ),
//             texture_atlas: world.resource_mut::<Assets<TextureAtlasLayout>>().add(
//                 TextureAtlasLayout::from_grid(UVec2::splat(TILE_SIZE as u32), 16, 16, None, None),
//             ),
//         }
//     }
// }

// /// Component that marks an entity as a tile with specific properties
// #[derive(Component, Reflect, Clone)]
// #[reflect(Component)]
// pub struct Tile {
//     pub tile_type: TileType,
//     pub is_watered: bool,
//     pub is_occupied: bool, // Has crop, obstacle, etc.
// }

// /// Component representing a tile's position in grid coordinates
// #[derive(Component, Reflect, Clone, Copy, PartialEq, Eq, Hash, Debug)]
// #[reflect(Component)]
// pub struct TilePosition {
//     pub x: i32,
//     pub y: i32,
// }

// /// Different types of ground tiles
// #[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Eq)]
// #[reflect(Component)]
// pub enum TileType {
//     Grass,
//     Dirt,
//     Tilled,
//     Water,
//     Sand,
//     Stone,
//     Path,
// }

fn spawn_tile_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let map_handle = super::tiledhelper::TiledMapHandle(asset_server.load("tilemaps/farm.tmx"));

    commands.spawn(super::tiledhelper::TiledMapBundle {
        tiled_map: map_handle,
        ..Default::default()
    });
}

// Spawn the initial map
// pub fn spawn_map(mut commands: Commands, tile_asset: Res<TileAssets>) {
//     // Spawn the map entity
//     let map_entity = commands
//         .spawn((
//             Name::new("Farm Map"),
//             TileMap {
//                 width: MAP_WIDTH,
//                 height: MAP_HEIGHT,
//                 tile_size: TILE_SIZE,
//                 tiles: HashMap::new(),
//             },
//             Transform::default(),
//             Visibility::default(),
//             StateScoped(Screen::Gameplay),
//         ))
//         .id();

//     // Generate tile distribution (you can replace this with your own map generation logic)
//     let mut tiles = HashMap::new();

//     // Basic farm layout with grass in the middle, water on edges
//     // Create a water border
//     // Center the map around the origin
//     let half_width = MAP_WIDTH / 2;
//     let half_height = MAP_HEIGHT / 2;

//     for y in -half_height..half_height {
//         for x in -half_width..half_width {
//             // Start with default tile type as grass
//             let mut tile_type = TileType::Grass;

//             // Water border (3 tiles wide)
//             if x < -half_width + 3
//                 || x >= half_width - 3
//                 || y < -half_height + 3
//                 || y >= half_height - 3
//             {
//                 tile_type = TileType::Water;
//             }
//             // Create a path from the center to the edge
//             else if (x == 0 || x == -1) && y >= -half_height + 3 {
//                 tile_type = TileType::Path;
//             }
//             // Create a dirt farm plot in the center
//             else if x > -half_width / 2
//                 && x < half_width / 2
//                 && y > -half_height / 2
//                 && y < half_height / 2
//             {
//                 // Some tilled soil for farming
//                 if ((x / 4) + (y / 4)) % 2 == 0 {
//                     tile_type = TileType::Tilled;
//                 } else {
//                     tile_type = TileType::Dirt;
//                 }
//             }
//             // Add some stone patches
//             else if (x as f32 * 0.7 + y as f32 * 0.3).sin() > 0.7 {
//                 tile_type = TileType::Stone;
//             }
//             // Add some sandy areas near water
//             else if (x < -half_width + 6
//                 || x >= half_width - 6
//                 || y < -half_height + 6
//                 || y >= half_height - 6)
//                 && (x + y) % 4 == 0
//             {
//                 tile_type = TileType::Sand;
//             }

//             tiles.insert(TilePosition { x, y }, tile_type);
//         }
//     }
//     // Spawn all the tiles
//     for (pos, tile_type) in &tiles {
//         spawn_tile(&mut commands, map_entity, *pos, &tile_asset, *tile_type);
//     }
// }

// /// Spawn a single tile
// fn spawn_tile(
//     commands: &mut Commands,
//     parent: Entity,
//     position: TilePosition,
//     tile_asset: &TileAssets,
//     tile_type: TileType,
// ) {
//     let tile_index = match tile_type {
//         TileType::Grass => 0,
//         TileType::Dirt => 1,
//         TileType::Tilled => 2,
//         TileType::Water => 3,
//         TileType::Sand => 4,
//         TileType::Stone => 5,
//         TileType::Path => 6,
//     };

//     let tile_entity = commands
//         .spawn((
//             Name::new(format!("Tile ({}, {})", position.x, position.y)),
//             Tile {
//                 tile_type,
//                 is_watered: false,
//                 is_occupied: false,
//             },
//             Sprite {
//                 image: tile_asset.asset.clone(),
//                 texture_atlas: Some(TextureAtlas {
//                     layout: tile_asset.texture_atlas.clone(),
//                     index: tile_index,
//                 }),
//                 ..default()
//             },
//             Transform::from_translation(tile_to_world(position).extend(0.0))
//                 .with_scale(Vec3::new(TILE_SCALE, TILE_SCALE, 1.0)),
//         ))
//         .id();

//     // Add the tile as a child of the map
//     commands.entity(parent).add_child(tile_entity);
// }

// // /// Convert world position to tile coordinates
// // pub fn world_to_tile(world_pos: Vec2) -> TilePosition {
// //     TilePosition {
// //         x: (world_pos.x / TILE_SIZE).floor() as i32,
// //         y: (world_pos.y / TILE_SIZE).floor() as i32,
// //     }
// // }
// /// Convert tile coordinates to world position (center of tile)
// pub fn tile_to_world(tile_pos: TilePosition) -> Vec2 {
//     Vec2::new(
//         (tile_pos.x * TILE_SIZE) as f32 * TILE_SCALE,
//         (tile_pos.y * TILE_SIZE) as f32 * TILE_SCALE,
//     )
// }
