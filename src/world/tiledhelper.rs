// How to use this:
//   You should copy/paste this into your project and use it much like examples/tiles.rs uses this
//   file. When you do so you will need to adjust the code based on whether you're using the
//   'atlas` feature in bevy_ecs_tilemap. The bevy_ecs_tilemap uses this as an example of how to
//   use both single image tilesets and image collection tilesets. Since your project won't have
//   the 'atlas' feature defined in your Cargo config, the expressions prefixed by the #[cfg(...)]
//   macro will not compile in your project as-is. If your project depends on the bevy_ecs_tilemap
//   'atlas' feature then move all of the expressions prefixed by .
//   Otherwise remove all of the expressions prefixed by #[cfg(feature = "atlas")].
//
// Functional limitations:
//   * When the 'atlas' feature is enabled tilesets using a collection of images will be skipped.
//   * Only finite tile layers are loaded. Infinite tile layers and object layers will be skipped.

use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;

use bevy::input::ButtonState;
use bevy::input::mouse::MouseButtonInput;
use bevy::log::{info, warn};
use bevy::prelude::*;
use bevy::reflect::Reflect;
use bevy::{
    asset::{AssetLoader, AssetPath, io::Reader},
    platform::collections::HashMap,
    reflect::TypePath,
};
use bevy_ecs_tilemap::prelude::*;
use thiserror::Error;

use crate::AppSystems;
use crate::constants::TILE_SCALE;
use crate::game::camera::CursorPos;

#[derive(Default)]
pub struct TiledPlugin;

impl Plugin for TiledPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_asset::<TiledMap>()
            .register_asset_loader(TiledLoader)
            .register_type::<TileType>()
            .add_systems(
                Update,
                (
                    process_loaded_maps,
                    (handle_mouse_highlight, apply_highlight_effect)
                        .run_if(on_event::<MouseButtonInput>),
                )
                    .chain()
                    .in_set(AppSystems::PreUpdate),
            );
    }
}

#[derive(Component, Debug, Clone, Reflect)]
pub enum TileType {
    Grass,
    Dirt,
    Water,
    Sand,
    Rock,
}

#[derive(Component)]
struct HighlightedTile;

#[derive(TypePath, Asset)]
pub struct TiledMap {
    pub map: tiled::Map,

    pub tilemap_textures: HashMap<usize, TilemapTexture>,

    // The offset into the tileset_images for each tile id within each tileset.
    pub tile_image_offsets: HashMap<(usize, tiled::TileId), u32>,
}

// Stores a list of tiled layers.
#[derive(Component, Default)]
pub struct TiledLayersStorage {
    pub storage: HashMap<u32, Entity>,
}

#[derive(Component, Default)]
pub struct TiledMapHandle(pub Handle<TiledMap>);

#[derive(Default, Bundle)]
pub struct TiledMapBundle {
    pub tiled_map: TiledMapHandle,
    pub storage: TiledLayersStorage,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub render_settings: TilemapRenderSettings,
}

struct BytesResourceReader {
    bytes: Arc<[u8]>,
}

impl BytesResourceReader {
    fn new(bytes: &[u8]) -> Self {
        Self {
            bytes: Arc::from(bytes),
        }
    }
}

impl tiled::ResourceReader for BytesResourceReader {
    type Resource = Cursor<Arc<[u8]>>;
    type Error = std::io::Error;

    fn read_from(&mut self, _path: &Path) -> std::result::Result<Self::Resource, Self::Error> {
        // In this case, the path is ignored because the byte data is already provided.
        Ok(Cursor::new(self.bytes.clone()))
    }
}

pub struct TiledLoader;

#[derive(Debug, Error)]
pub enum TiledAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load Tiled file: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for TiledLoader {
    type Asset = TiledMap;
    type Settings = ();
    type Error = TiledAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let mut loader = tiled::Loader::with_cache_and_reader(
            tiled::DefaultResourceCache::new(),
            BytesResourceReader::new(&bytes),
        );
        let map = loader
            .load_tmx_map(load_context.path())
            .map_err(|e| std::io::Error::other(format!("Could not load TMX map: {e}")))?;

        let mut tilemap_textures = HashMap::default();

        let mut tile_image_offsets = HashMap::default();

        for (tileset_index, tileset) in map.tilesets().iter().enumerate() {
            let tilemap_texture = match &tileset.image {
                None => {
                    {
                        let mut tile_images: Vec<Handle<Image>> = Vec::new();
                        for (tile_id, tile) in tileset.tiles() {
                            if let Some(img) = &tile.image {
                                // The load context path is the TMX file itself. If the file is at the root of the
                                // assets/ directory structure then the tmx_dir will be empty, which is fine.
                                let tmx_dir = load_context
                                    .path()
                                    .parent()
                                    .expect("The asset load context was empty.");
                                let tile_path = tmx_dir.join(&img.source);
                                let asset_path = AssetPath::from(tile_path);
                                info!(
                                    "Loading tile image from {asset_path:?} as image ({tileset_index}, {tile_id})"
                                );
                                let texture: Handle<Image> = load_context.load(asset_path.clone());
                                tile_image_offsets
                                    .insert((tileset_index, tile_id), tile_images.len() as u32);
                                tile_images.push(texture.clone());
                            }
                        }

                        TilemapTexture::Vector(tile_images)
                    }
                }
                Some(img) => {
                    // The load context path is the TMX file itself. If the file is at the root of the
                    // assets/ directory structure then the tmx_dir will be empty, which is fine.
                    // let tmx_dir = load_context
                    //     .path()
                    //     .parent()
                    //     .expect("The asset load context was empty.");
                    let tile_path = img.source.clone();
                    let asset_path = AssetPath::from(tile_path);
                    let texture: Handle<Image> = load_context.load(asset_path.clone());

                    TilemapTexture::Single(texture.clone())
                }
            };

            tilemap_textures.insert(tileset_index, tilemap_texture);
        }

        let asset_map = TiledMap {
            map,
            tilemap_textures,

            tile_image_offsets,
        };

        info!("Loaded map: {}", load_context.path().display());
        Ok(asset_map)
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["tmx"];
        EXTENSIONS
    }
}

pub fn process_loaded_maps(
    mut commands: Commands,
    mut map_events: EventReader<AssetEvent<TiledMap>>,
    maps: Res<Assets<TiledMap>>,
    tile_storage_query: Query<(Entity, &TileStorage)>,
    mut map_query: Query<(
        &TiledMapHandle,
        &mut TiledLayersStorage,
        &TilemapRenderSettings,
    )>,
    new_maps: Query<&TiledMapHandle, Added<TiledMapHandle>>,
) {
    let mut changed_maps = Vec::<AssetId<TiledMap>>::default();
    for event in map_events.read() {
        match event {
            AssetEvent::Added { id } => {
                info!("Map added!");
                changed_maps.push(*id);
            }
            AssetEvent::Modified { id } => {
                info!("Map changed!");
                changed_maps.push(*id);
            }
            AssetEvent::Removed { id } => {
                info!("Map removed!");
                // if mesh was modified and removed in the same update, ignore the modification
                // events are ordered so future modification events are ok
                changed_maps.retain(|changed_handle| changed_handle == id);
            }
            _ => continue,
        }
    }

    // If we have new map entities add them to the changed_maps list.
    for new_map_handle in new_maps.iter() {
        changed_maps.push(new_map_handle.0.id());
    }

    for changed_map in changed_maps.iter() {
        for (map_handle, mut layer_storage, render_settings) in map_query.iter_mut() {
            // only deal with currently changed map
            if map_handle.0.id() != *changed_map {
                continue;
            }
            if let Some(tiled_map) = maps.get(&map_handle.0) {
                // TODO: Create a RemoveMap component..
                for layer_entity in layer_storage.storage.values() {
                    if let Ok((_, layer_tile_storage)) = tile_storage_query.get(*layer_entity) {
                        for tile in layer_tile_storage.iter().flatten() {
                            commands.entity(*tile).despawn()
                        }
                    }
                    // commands.entity(*layer_entity).despawn_recursive();
                }

                // The TilemapBundle requires that all tile images come exclusively from a single
                // tiled texture or from a Vec of independent per-tile images. Furthermore, all of
                // the per-tile images must be the same size. Since Tiled allows tiles of mixed
                // tilesets on each layer and allows differently-sized tile images in each tileset,
                // this means we need to load each combination of tileset and layer separately.
                for (tileset_index, tileset) in tiled_map.map.tilesets().iter().enumerate() {
                    let Some(tilemap_texture) = tiled_map.tilemap_textures.get(&tileset_index)
                    else {
                        warn!("Skipped creating layer with missing tilemap textures.");
                        continue;
                    };

                    let tile_size = TilemapTileSize {
                        x: tileset.tile_width as f32,
                        y: tileset.tile_height as f32,
                    };

                    let tile_spacing = TilemapSpacing {
                        x: tileset.spacing as f32,
                        y: tileset.spacing as f32,
                    };

                    // Once materials have been created/added we need to then create the layers.
                    for (layer_index, layer) in tiled_map.map.layers().enumerate() {
                        let offset_x = layer.offset_x;
                        let offset_y = layer.offset_y;

                        let tiled::LayerType::Tiles(tile_layer) = layer.layer_type() else {
                            info!(
                                "Skipping layer {} because only tile layers are supported.",
                                layer.id()
                            );
                            continue;
                        };

                        let tiled::TileLayer::Finite(layer_data) = tile_layer else {
                            info!(
                                "Skipping layer {} because only finite layers are supported.",
                                layer.id()
                            );
                            continue;
                        };

                        let map_size = TilemapSize {
                            x: tiled_map.map.width,
                            y: tiled_map.map.height,
                        };

                        let grid_size = TilemapGridSize {
                            x: tiled_map.map.tile_width as f32,
                            y: tiled_map.map.tile_height as f32,
                        };

                        let map_type = match tiled_map.map.orientation {
                            tiled::Orientation::Hexagonal => {
                                TilemapType::Hexagon(HexCoordSystem::Row)
                            }
                            tiled::Orientation::Isometric => {
                                TilemapType::Isometric(IsoCoordSystem::Diamond)
                            }
                            tiled::Orientation::Staggered => {
                                TilemapType::Isometric(IsoCoordSystem::Staggered)
                            }
                            tiled::Orientation::Orthogonal => TilemapType::Square,
                        };

                        let mut tile_storage = TileStorage::empty(map_size);
                        let layer_entity = commands.spawn_empty().id();

                        for x in 0..map_size.x {
                            for y in 0..map_size.y {
                                // Transform TMX coords into bevy coords.
                                let mapped_y = tiled_map.map.height - 1 - y;

                                let mapped_x = x as i32;
                                let mapped_y = mapped_y as i32;

                                let layer_tile = match layer_data.get_tile(mapped_x, mapped_y) {
                                    Some(t) => t,
                                    None => {
                                        continue;
                                    }
                                };
                                if tileset_index != layer_tile.tileset_index() {
                                    continue;
                                }
                                let layer_tile_data =
                                    match layer_data.get_tile_data(mapped_x, mapped_y) {
                                        Some(d) => d,
                                        None => {
                                            continue;
                                        }
                                    };

                                // Get tile properties from the tileset
                                let tile_properties: HashMap<String, tiled::PropertyValue> =
                                    if let Some(tileset) =
                                        tiled_map.map.tilesets().get(tileset_index)
                                    {
                                        if let Some(tile_def) = tileset.get_tile(layer_tile.id()) {
                                            // Convert std::collections::HashMap to bevy::utils::HashMap
                                            HashMap::from_iter(tile_def.properties.clone())
                                        } else {
                                            HashMap::default()
                                        }
                                    } else {
                                        HashMap::default()
                                    };

                                let texture_index = match tilemap_texture {
                                    TilemapTexture::Single(_) => layer_tile.id(),
                                    TilemapTexture::Vector(_) =>
                                        *tiled_map.tile_image_offsets.get(&(tileset_index, layer_tile.id()))
                                        .expect("The offset into to image vector should have been saved during the initial load."),
                                    _ => unreachable!()
                                };

                                let tile_pos = TilePos { x, y };
                                let tile_entity = commands
                                    .spawn((
                                        TileBundle {
                                            position: tile_pos,
                                            tilemap_id: TilemapId(layer_entity),
                                            texture_index: TileTextureIndex(texture_index),
                                            flip: TileFlip {
                                                x: layer_tile_data.flip_h,
                                                y: layer_tile_data.flip_v,
                                                d: layer_tile_data.flip_d,
                                            },
                                            ..Default::default()
                                        },
                                        // Add the tile properties component
                                        Name::new(format!(
                                            "Tile ({}, {}, {})",
                                            tile_pos.x, tile_pos.y, layer_index
                                        )),
                                    ))
                                    .id();
                                if tile_properties.get("type").is_none() {
                                    warn!("Tile type are empty for tile id {}", layer_tile.id());
                                } else if let Some(tile_type_value) = tile_properties.get("type") {
                                    let tile_type = match tile_type_value {
                                        tiled::PropertyValue::StringValue(s) => match s.as_str() {
                                            "Grass" => TileType::Grass,
                                            "Dirt" => TileType::Dirt,
                                            "Water" => TileType::Water,
                                            "Sand" => TileType::Sand,
                                            "Rock" => TileType::Rock,
                                            _ => TileType::Grass,
                                        },
                                        _ => {
                                            panic!(
                                                "Tile type is not a valid string for tile id {}",
                                                layer_tile.id()
                                            );
                                        }
                                    };
                                    commands.entity(tile_entity).insert(tile_type);
                                }
                                tile_storage.set(&tile_pos, tile_entity);
                            }
                        }

                        commands.entity(layer_entity).insert(TilemapBundle {
                            grid_size,
                            size: map_size,
                            storage: tile_storage,
                            texture: tilemap_texture.clone(),
                            tile_size,
                            spacing: tile_spacing,
                            anchor: TilemapAnchor::Center,
                            transform: Transform::from_xyz(offset_x, -offset_y, layer_index as f32)
                                .with_scale(Vec2::splat(TILE_SCALE).extend(1.0)),
                            map_type,
                            render_settings: *render_settings,

                            ..Default::default()
                        });

                        layer_storage
                            .storage
                            .insert(layer_index as u32, layer_entity);
                    }
                }
            }
        }
    }
}

fn apply_highlight_effect(
    mut highlighted_tiles_q: Query<&mut TileVisible, Added<HighlightedTile>>,
) {
    for mut visible in highlighted_tiles_q.iter_mut() {
        visible.0 = !visible.0;
    }
}
fn handle_mouse_highlight(
    mut commands: Commands,
    cursor_pos: Res<CursorPos>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapTileSize,
        &TilemapType,
        &TileStorage,
        &Transform,
        &TilemapAnchor,
    )>,
    highlighted_tiles_q: Query<Entity, With<HighlightedTile>>,
) {
    let mut clicked = false;
    for event in mouse_button_input_events.read() {
        if event.button == MouseButton::Left && event.state == ButtonState::Pressed {
            clicked = true;
            break;
        }
    }
    if !clicked {
        return;
    }
    for highlighted_tile_entity in highlighted_tiles_q.iter() {
        commands
            .entity(highlighted_tile_entity)
            .remove::<HighlightedTile>();
    }

    for (map_size, grid_size, tile_size, map_type, tile_storage, map_transform, anchor) in
        tilemap_q.iter()
    {
        let cursor_pos: Vec2 = cursor_pos.0;
        let cursor_in_map_pos: Vec2 = {
            let cursor_pos = Vec4::from((cursor_pos, 0.0, 1.0));
            let cursor_in_map_pos = map_transform.compute_matrix().inverse() * cursor_pos;
            cursor_in_map_pos.xy()
        };
        if let Some(tile_pos) = TilePos::from_world_pos(
            &cursor_in_map_pos,
            map_size,
            grid_size,
            tile_size,
            map_type,
            anchor,
        ) {
            if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                commands.entity(tile_entity).insert(HighlightedTile);
            }
        }
    }
}
