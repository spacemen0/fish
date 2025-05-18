//! A loading screen during which game assets are loaded.
//! This reduces stuttering, especially for audio on WASM.

use bevy::prelude::*;

use crate::{
    asset_tracking::ResourceHandles,
    states::{GameState, PreviousState},
    theme::prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Loading), spawn_loading_screen);

    app.add_systems(
        Update,
        enter_gameplay_screen.run_if(in_state(GameState::Loading).and(all_assets_loaded)),
    );
}

fn spawn_loading_screen(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Loading Screen"),
        StateScoped(GameState::Loading),
        children![widget::label("Loading...")],
    ));
}

fn enter_gameplay_screen(
    mut next_screen: ResMut<NextState<GameState>>,
    mut previous_state: ResMut<PreviousState>,
) {
    previous_state.0 = GameState::Loading;
    next_screen.set(GameState::Gameplay);
}

fn all_assets_loaded(resource_handles: Res<ResourceHandles>) -> bool {
    resource_handles.is_all_done()
}
