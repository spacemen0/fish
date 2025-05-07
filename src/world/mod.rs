pub mod tiledhelper;
pub mod tilemap;
use bevy::prelude::*;
use tiledhelper::TiledPlugin;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        tilemap::plugin,
        bevy_ecs_tilemap::TilemapPlugin,
        TiledPlugin,
    ));
}
