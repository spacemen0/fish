use crate::asset_tracking::LoadResource;
use crate::constants::*;
use crate::states::VisibleInState;
use crate::{
    AppSystems,
    game::{camera::WithinBounds, movement::MovementController},
    states::GameState,
};
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;
pub(super) fn plugin(app: &mut App) {
    app.register_type::<EnemyAssets>();
    app.load_resource::<EnemyAssets>();
    app.add_systems(OnEnter(GameState::Gameplay), spawn_enemies);
    app.add_systems(
        Update,
        tick_enemy_roaming_timers
            .in_set(AppSystems::TickTimers)
            .run_if(in_state(GameState::Gameplay)),
    );
    app.add_systems(
        Update,
        apply_roaming
            .in_set(AppSystems::Update)
            .run_if(in_state(GameState::Gameplay)),
    );
}

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub(crate) struct Enemy {
    pub roaming_timer: Timer,
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            roaming_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
        }
    }
}

pub fn enemy(
    player_assets: &EnemyAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    pos: &Vec2,
) -> impl Bundle {
    // A texture atlas is a way to split a single image into a grid of related images.
    // You can learn more in this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
    let layout =
        TextureAtlasLayout::from_grid(UVec2::new(GRID_SIZE_X, GRID_SIZE_Y), 3, 2, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    (
        Enemy::default(),
        Sprite {
            image: player_assets.enemies.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: 0,
            }),
            ..default()
        },
        Transform::from_translation(pos.extend(PLAYER_Z))
            .with_scale(Vec2::splat(PLAYER_SCALE).extend(1.0)),
        MovementController {
            max_speed: PLAYER_MAX_SPEED / 8.0,
            ..default()
        },
        WithinBounds,
        bevy::camera::primitives::Aabb::from_min_max(
            Vec3::new(-8.0, -8.0, 0.0),
            Vec3::new(8.0, 8.0, 0.0),
        ),
    )
}

fn spawn_enemies(
    mut commands: Commands,
    enemy_assets: Res<EnemyAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Spawn a few enemies at random positions.
    for i in 0..5 {
        let position = Vec2::new(100.0 * (i as f32 + 1.0), 100.0);
        commands.spawn((
            Name::new(format!("Enemy {i}")),
            enemy(&enemy_assets, &mut texture_atlas_layouts, &position),
            VisibleInState(vec![GameState::Gameplay]),
        ));
    }
}

fn tick_enemy_roaming_timers(time: Res<Time>, mut enemy_query: Query<&mut Enemy>) {
    for mut enemy in &mut enemy_query {
        enemy.roaming_timer.tick(time.delta());
    }
}

fn apply_roaming(mut enemy_query: Query<(&Enemy, &mut MovementController)>) {
    for (enemy, mut controller) in &mut enemy_query {
        if enemy.roaming_timer.just_finished() {
            controller.intent = Vec2::new(
                rand::random::<f32>() * 2.0 - 1.0,
                rand::random::<f32>() * 2.0 - 1.0,
            )
            .normalize_or_zero();
        }
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct EnemyAssets {
    #[dependency]
    enemies: Handle<Image>,
}

impl FromWorld for EnemyAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            enemies: assets.load_with_settings(
                "images/enemies.png",
                |settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::nearest();
                },
            ),
        }
    }
}
