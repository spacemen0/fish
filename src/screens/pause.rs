use bevy::prelude::*;

use crate::{
    states::{GameState, PreviousState},
    theme::widget,
};

use super::title::enter_settings_screen;
#[cfg(not(target_family = "wasm"))]
use super::title::exit_app;
pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Pausing), spawn_pausing_screen);
}
fn spawn_pausing_screen(mut commands: Commands) {
    #[cfg(not(target_family = "wasm"))]
    commands
        .spawn_scene(bsn! {
            widget::ui_root("Pausing Screen")
            Children [
                widget::button("Continue", continue_to_gameplay_screen),
                widget::button("Settings", enter_settings_screen),
                widget::button("Title", enter_title_screen),
                widget::button("Exit", exit_app),
            ]
        })
        .insert(DespawnOnExit(GameState::Pausing));
    #[cfg(target_family = "wasm")]
    commands
        .spawn_scene(bsn! {
            widget::ui_root("Pausing Screen")
            Children [
                widget::button("Continue", continue_to_gameplay_screen),
                widget::button("Settings", enter_settings_screen),
                widget::button("Title", enter_title_screen),
            ]
        })
        .insert(DespawnOnExit(GameState::Pausing));
}

fn continue_to_gameplay_screen(
    _: On<Pointer<Click>>,
    mut next_screen: ResMut<NextState<GameState>>,
    mut previous_state: ResMut<PreviousState>,
) {
    previous_state.0 = GameState::Pausing;
    next_screen.set(GameState::Gameplay);
}

fn enter_title_screen(
    _: On<Pointer<Click>>,
    mut next_screen: ResMut<NextState<GameState>>,
    mut previous_state: ResMut<PreviousState>,
) {
    previous_state.0 = GameState::Pausing;
    next_screen.set(GameState::Title);
}
