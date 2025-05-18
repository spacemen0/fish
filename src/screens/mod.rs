//! The game's main screen states and transitions between them.

mod credits;
mod gameplay;
mod loading;
mod pause;
mod settings;
mod splash;
mod title;

use bevy::prelude::*;

use crate::states::GameState;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<GameState>();
    app.enable_state_scoped_entities::<GameState>();

    app.add_plugins((
        credits::plugin,
        gameplay::plugin,
        loading::plugin,
        settings::plugin,
        splash::plugin,
        title::plugin,
        pause::plugin,
    ));
}
