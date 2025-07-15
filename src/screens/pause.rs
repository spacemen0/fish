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
    commands.spawn((
        widget::ui_root("Pausing Screen"),
        StateScoped(GameState::Pausing),
        #[cfg(not(target_family = "wasm"))]
        children![
            widget::button("Continue", continue_to_gameplay_screen),
            widget::button("Settings", enter_settings_screen),
            widget::button("Title", enter_title_screen),
            widget::button("Exit", exit_app),
        ],
        #[cfg(target_family = "wasm")]
        children![
            widget::button("Continue", continue_to_gameplay_screen),
            widget::button("Settings", enter_settings_screen),
            widget::button("Title", enter_title_screen),
        ],
    ));
}

fn continue_to_gameplay_screen(
    _: Trigger<Pointer<Click>>,
    mut next_screen: ResMut<NextState<GameState>>,
    mut previous_state: ResMut<PreviousState>,
) {
    previous_state.0 = GameState::Pausing;
    next_screen.set(GameState::Gameplay);
}

fn enter_title_screen(
    _: Trigger<Pointer<Click>>,
    mut next_screen: ResMut<NextState<GameState>>,
    mut previous_state: ResMut<PreviousState>,
) {
    previous_state.0 = GameState::Pausing;
    next_screen.set(GameState::Title);
}
