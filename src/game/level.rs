//! Spawn the main level.

use bevy::prelude::*;

use crate::{
    game::player::{PlayerAssets, player},
    states::{DestroyOnEnter, GameState, VisibleInState},
};

/// A system that spawns the main level.
pub fn spawn_level(
    mut commands: Commands,
    player_assets: Res<PlayerAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn((
        Name::new("Level"),
        Transform::default(),
        Visibility::default(),
        VisibleInState(vec![GameState::Gameplay]),
        DestroyOnEnter(vec![GameState::Title]),
        children![player(&player_assets, &mut texture_atlas_layouts)],
    ));
}
