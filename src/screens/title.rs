//! The title screen that appears when the game starts.

use bevy::prelude::*;

use crate::{
    asset_tracking::ResourceHandles,
    states::{GameState, PreviousState},
    theme::prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Title), spawn_title_screen);
}

fn spawn_title_screen(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Title Screen"),
        StateScoped(GameState::Title),
        #[cfg(not(target_family = "wasm"))]
        children![
            widget::button("Play", enter_loading_or_gameplay_screen),
            widget::button("Settings", enter_settings_screen),
            widget::button("Credits", enter_credits_screen),
            widget::button("Exit", exit_app),
        ],
        #[cfg(target_family = "wasm")]
        children![
            widget::button("Play", enter_loading_or_gameplay_screen),
            widget::button("Settings", enter_settings_screen),
            widget::button("Credits", enter_credits_screen),
        ],
    ));
}

fn enter_loading_or_gameplay_screen(
    _: Trigger<Pointer<Click>>,
    resource_handles: Res<ResourceHandles>,
    mut next_screen: ResMut<NextState<GameState>>,
) {
    if resource_handles.is_all_done() {
        next_screen.set(GameState::Gameplay);
    } else {
        next_screen.set(GameState::Loading);
    }
}

pub fn enter_settings_screen(
    _: Trigger<Pointer<Click>>,
    mut next_screen: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
    mut previous_state: ResMut<PreviousState>,
) {
    previous_state.0 = current_state.get().clone();
    next_screen.set(GameState::Settings);
}

fn enter_credits_screen(
    _: Trigger<Pointer<Click>>,
    mut next_screen: ResMut<NextState<GameState>>,
    mut previous_state: ResMut<PreviousState>,
) {
    previous_state.0 = GameState::Title;
    next_screen.set(GameState::Credits);
}
#[cfg(not(target_family = "wasm"))]
pub fn exit_app(_: Trigger<Pointer<Click>>, mut app_exit: EventWriter<AppExit>) {
    app_exit.write(AppExit::Success);
}
