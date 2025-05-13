use bevy::prelude::*;

use crate::{AppSet, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(Screen::Gameplay),
        spawn_tile_map.in_set(AppSet::PreUpdate),
    );
}

fn spawn_tile_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let map_handle = super::tiledhelper::TiledMapHandle(asset_server.load("tilemaps/farm.tmx"));

    commands.spawn(super::tiledhelper::TiledMapBundle {
        tiled_map: map_handle,
        ..Default::default()
    });
}
