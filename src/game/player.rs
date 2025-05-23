//! Player-specific behavior.

use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};

use crate::{
    AppSystems,
    asset_tracking::LoadResource,
    constants::{GRID_SIZE_X, GRID_SIZE_Y},
    game::{animation::PlayerAnimation, movement::MovementController},
    states::GameState,
};

use crate::constants::{PLAYER_MAX_SPEED, PLAYER_SCALE, PLAYER_Z};

use super::{
    animation::{ActionType, PlayerActionState},
    camera::WithinBounds,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Player>();

    app.register_type::<PlayerAssets>();
    app.load_resource::<PlayerAssets>();

    // Record directional input as movement controls.
    app.add_systems(
        Update,
        (record_player_directional_input, record_player_actions_input)
            .chain()
            .run_if(in_state(GameState::Gameplay))
            .in_set(AppSystems::RecordInput),
    );
}

/// The player character.
pub fn player(
    player_assets: &PlayerAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> impl Bundle {
    // A texture atlas is a way to split a single image into a grid of related images.
    // You can learn more in this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
    let layout =
        TextureAtlasLayout::from_grid(UVec2::new(GRID_SIZE_X, GRID_SIZE_Y), 16, 6, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let player_animation = PlayerAnimation::new();

    (
        Name::new("Player"),
        Player,
        Sprite {
            image: player_assets.player.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: player_animation.get_atlas_index(),
            }),
            ..default()
        },
        Transform::from_translation(Vec2::splat(0.0).extend(PLAYER_Z))
            .with_scale(Vec2::splat(PLAYER_SCALE).extend(1.0)),
        MovementController {
            max_speed: PLAYER_MAX_SPEED,
            ..default()
        },
        WithinBounds,
        player_animation,
        PlayerActionState::default(),
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub(crate) struct Player;

fn record_player_directional_input(
    input: Res<ButtonInput<KeyCode>>,
    mut controller_query: Query<&mut MovementController, With<Player>>,
) {
    // Collect directional input.
    let mut intent = Vec2::ZERO;
    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
        intent.y += 1.0;
    }
    if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
        intent.y -= 1.0;
    }
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        intent.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        intent.x += 1.0;
    }

    let intent = intent.normalize_or_zero();

    for mut controller in &mut controller_query {
        controller.intent = intent;
    }
}

fn record_player_actions_input(
    input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut PlayerActionState, &MovementController)>,
) {
    let (mut action_state, controller) = player_query.single_mut().expect("Player should exist!");

    if action_state.current_action.is_none() {
        // Only allow starting actions when not moving
        if controller.intent == Vec2::ZERO {
            if input.just_pressed(KeyCode::KeyE) {
                action_state.current_action = Some(ActionType::Watering);
                action_state.action_progress = 0.0;
            } else if input.just_pressed(KeyCode::KeyQ) {
                action_state.current_action = Some(ActionType::Hoeing);
                action_state.action_progress = 0.0;
            } else if input.just_pressed(KeyCode::KeyF) {
                action_state.current_action = Some(ActionType::Chopping);
                action_state.action_progress = 0.0;
            }
        }
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[dependency]
    player: Handle<Image>,
    #[dependency]
    pub steps: Vec<Handle<AudioSource>>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            player: assets.load_with_settings(
                "images/character.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            steps: vec![
                assets.load("audio/sound_effects/step1.ogg"),
                assets.load("audio/sound_effects/step2.ogg"),
                assets.load("audio/sound_effects/step3.ogg"),
                assets.load("audio/sound_effects/step4.ogg"),
            ],
        }
    }
}
