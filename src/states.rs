use bevy::prelude::*;

use crate::AppSystems;
pub(super) fn plugin(app: &mut App) {
    app.register_type::<VisibleInState>()
        .register_type::<DestroyOnEnter>()
        .init_resource::<PreviousState>()
        .add_systems(
            Update,
            (visible_in_state, destroy_on_enter)
                .run_if(state_changed::<GameState>)
                .in_set(AppSystems::PreUpdate),
        );
}
#[derive(Component, Clone, Reflect)]
pub struct DestroyOnEnter(pub Vec<GameState>);

#[derive(Resource, Clone, Reflect)]
pub struct PreviousState(pub GameState);

impl FromWorld for PreviousState {
    fn from_world(world: &mut World) -> Self {
        let state = world.resource::<State<GameState>>();
        Self(state.get().clone())
    }
}

fn destroy_on_enter(
    mut commands: Commands,
    query: Query<(Entity, &DestroyOnEnter)>,
    current_state: Res<State<GameState>>,
) {
    for (entity, destroy_on) in query.iter() {
        if destroy_on.0.contains(&*current_state) {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Component, Clone, Reflect)]
pub struct VisibleInState(pub Vec<GameState>);

fn visible_in_state(
    mut query: Query<(&mut Visibility, &VisibleInState)>,
    current_state: Res<State<GameState>>,
) {
    for (mut visibility, state) in query.iter_mut() {
        if state.0.contains(&*current_state) {
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

/// The game's main screen states.
#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default, Reflect)]
pub enum GameState {
    #[default]
    Splash,
    Title,
    Credits,
    Settings,
    Loading,
    Pausing,
    Gameplay,
}
