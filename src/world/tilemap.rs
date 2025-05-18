use bevy::prelude::*;

use crate::{
    AppSystems,
    states::{DestroyOnEnter, GameState},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnTransition::<GameState> {
            exited: GameState::Title,
            entered: GameState::Gameplay,
        },
        spawn_tile_map.in_set(AppSystems::PreUpdate),
    );
}

fn spawn_tile_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let map_handle = super::tiledhelper::TiledMapHandle(asset_server.load("tilemaps/farm.tmx"));

    commands.spawn((
        super::tiledhelper::TiledMapBundle {
            tiled_map: map_handle,
            ..Default::default()
        },
        DestroyOnEnter(vec![GameState::Title]),
    ));
}
