//! The screen state for the main gameplay.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::{
    asset_tracking::LoadResource,
    audio::music,
    game::level::spawn_level,
    states::{GameState, PreviousState},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnTransition::<GameState> {
            exited: GameState::Title,
            entered: GameState::Gameplay,
        },
        spawn_level,
    );

    app.register_type::<GameplayMusic>();
    app.load_resource::<GameplayMusic>();
    app.add_systems(OnEnter(GameState::Gameplay), start_gameplay_music);
    app.add_systems(OnExit(GameState::Gameplay), stop_gameplay_music);

    app.add_systems(
        Update,
        pause_or_continue_gameplay.run_if(
            (in_state(GameState::Gameplay).or(in_state(GameState::Pausing)))
                .and(input_just_pressed(KeyCode::Escape)),
        ),
    );
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
struct GameplayMusic {
    #[dependency]
    handle: Handle<AudioSource>,
    entity: Option<Entity>,
}

impl FromWorld for GameplayMusic {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            handle: assets.load("audio/music/Fluffing A Duck.ogg"),
            entity: None,
        }
    }
}

fn start_gameplay_music(mut commands: Commands, mut gameplay_music: ResMut<GameplayMusic>) {
    let handle = gameplay_music.handle.clone();
    gameplay_music.entity = Some(commands.spawn(music(handle)).id());
}

fn stop_gameplay_music(mut commands: Commands, mut gameplay_music: ResMut<GameplayMusic>) {
    if let Some(entity) = gameplay_music.entity.take() {
        commands.entity(entity).despawn();
    }
}

fn pause_or_continue_gameplay(
    current_screen: Res<State<GameState>>,
    mut next_screen: ResMut<NextState<GameState>>,
    mut previous_state: ResMut<PreviousState>,
) {
    previous_state.0 = current_screen.get().clone();
    if current_screen.get() == &GameState::Pausing {
        next_screen.set(GameState::Gameplay);
        return;
    }
    next_screen.set(GameState::Pausing);
}
