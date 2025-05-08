use bevy::prelude::*;
use rand::prelude::*;
use std::time::Duration;

use crate::{
    AppSet,
    audio::sound_effect,
    demo::{movement::MovementController, player::PlayerAssets},
};

pub(super) fn plugin(app: &mut App) {
    // Animate and play sound effects based on controls.
    app.register_type::<PlayerAnimation>();
    app.add_systems(
        Update,
        (
            update_animation_timer.in_set(AppSet::TickTimers),
            (
                update_animation_movement,
                update_animation_atlas,
                trigger_step_sound_effect,
            )
                .chain()
                .run_if(resource_exists::<PlayerAssets>)
                .in_set(AppSet::Update),
        ),
    );
}

/// Update the sprite direction and animation state (idling/walking).
fn update_animation_movement(mut player_query: Query<(&MovementController, &mut PlayerAnimation)>) {
    for (controller, mut animation) in &mut player_query {
        // Determine animation state based on movement direction
        let intent = controller.intent;

        // Default to the current state (in case we don't change it)
        let animation_state = if intent == Vec2::ZERO {
            // Idle state - maintain the current direction when idle
            match animation.state {
                PlayerAnimationState::WalkingT | PlayerAnimationState::IdlingT => {
                    PlayerAnimationState::IdlingT
                }
                PlayerAnimationState::WalkingB | PlayerAnimationState::IdlingB => {
                    PlayerAnimationState::IdlingB
                }
                PlayerAnimationState::WalkingL | PlayerAnimationState::IdlingL => {
                    PlayerAnimationState::IdlingL
                }
                PlayerAnimationState::WalkingR | PlayerAnimationState::IdlingR => {
                    PlayerAnimationState::IdlingR
                }
            }
        } else {
            // Determine direction based on which component is larger
            if intent.y.abs() > intent.x.abs() {
                // Vertical movement is dominant
                if intent.y > 0.0 {
                    PlayerAnimationState::WalkingT
                } else {
                    PlayerAnimationState::WalkingB
                }
            } else {
                // Horizontal movement is dominant
                if intent.x > 0.0 {
                    PlayerAnimationState::WalkingR
                } else {
                    PlayerAnimationState::WalkingL
                }
            }
        };

        if animation.state != animation_state {
            //setting state changed to true in this method already
            animation.update_state(animation_state);
        }
    }
}

/// Update the animation timer.
fn update_animation_timer(time: Res<Time>, mut query: Query<&mut PlayerAnimation>) {
    for mut animation in &mut query {
        animation.update_timer(time.delta());
    }
}

/// Update the texture atlas to reflect changes in the animation.
fn update_animation_atlas(mut query: Query<(&mut PlayerAnimation, &mut Sprite)>) {
    for (mut animation, mut sprite) in &mut query {
        let Some(atlas) = sprite.texture_atlas.as_mut() else {
            continue;
        };
        if animation.changed() {
            atlas.index = animation.get_atlas_index();
        }
    }
}

// probably shouldnt use mut
/// If the player is moving, play a step sound effect synchronized with the
/// animation.
fn trigger_step_sound_effect(
    mut commands: Commands,
    player_assets: Res<PlayerAssets>,
    mut step_query: Query<&mut PlayerAnimation>,
) {
    for mut animation in &mut step_query {
        if animation.state == PlayerAnimationState::WalkingB
            && animation.changed()
            && (animation.frame == 2 || animation.frame == 5)
        {
            let rng = &mut rand::thread_rng();
            let random_step = player_assets
                .steps
                .choose(rng)
                .expect("Player assets should exist!")
                .clone();
            commands.spawn(sound_effect(random_step));
            animation.set_state_changed(false);
        }
    }
}

/// Component that tracks player's animation state.
/// It is tightly bound to the texture atlas we use.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlayerAnimation {
    timer: Timer,
    frame: usize,
    state: PlayerAnimationState,
    state_changed: bool,
}

#[derive(Reflect, PartialEq, Debug)]
pub enum PlayerAnimationState {
    IdlingT,
    IdlingB,
    IdlingL,
    IdlingR,
    WalkingT,
    WalkingB,
    WalkingL,
    WalkingR,
}

impl PlayerAnimation {
    /// The number of idle frames.
    const IDLE_FRAMES: usize = 2;
    /// The duration of each idle frame.
    const IDLE_INTERVAL: Duration = Duration::from_millis(500);
    /// The number of walking frames.
    const WALKING_FRAMES: usize = 2;
    /// The duration of each walking frame.
    const WALKING_INTERVAL: Duration = Duration::from_millis(200);

    // Bottom facing animations (index 0-3)
    fn idling_bottom() -> Self {
        Self {
            timer: Timer::new(Self::IDLE_INTERVAL, TimerMode::Repeating),
            frame: 0,
            state: PlayerAnimationState::IdlingB,
            state_changed: true,
        }
    }

    fn walking_bottom() -> Self {
        Self {
            timer: Timer::new(Self::WALKING_INTERVAL, TimerMode::Repeating),
            frame: 0,
            state: PlayerAnimationState::WalkingB,
            state_changed: true,
        }
    }

    // Left facing animations (index 4-7)
    fn idling_left() -> Self {
        Self {
            timer: Timer::new(Self::IDLE_INTERVAL, TimerMode::Repeating),
            frame: 0,
            state: PlayerAnimationState::IdlingL,
            state_changed: true,
        }
    }

    fn walking_left() -> Self {
        Self {
            timer: Timer::new(Self::WALKING_INTERVAL, TimerMode::Repeating),
            frame: 0,
            state: PlayerAnimationState::WalkingL,
            state_changed: true,
        }
    }

    // Right facing animations (index 8-11)
    fn idling_right() -> Self {
        Self {
            timer: Timer::new(Self::IDLE_INTERVAL, TimerMode::Repeating),
            frame: 0,
            state: PlayerAnimationState::IdlingR,
            state_changed: true,
        }
    }

    fn walking_right() -> Self {
        Self {
            timer: Timer::new(Self::WALKING_INTERVAL, TimerMode::Repeating),
            frame: 0,
            state: PlayerAnimationState::WalkingR,
            state_changed: true,
        }
    }

    // Top facing animations (index 12-15)
    fn idling_top() -> Self {
        Self {
            timer: Timer::new(Self::IDLE_INTERVAL, TimerMode::Repeating),
            frame: 0,
            state: PlayerAnimationState::IdlingT,
            state_changed: true,
        }
    }

    fn walking_top() -> Self {
        Self {
            timer: Timer::new(Self::WALKING_INTERVAL, TimerMode::Repeating),
            frame: 0,
            state: PlayerAnimationState::WalkingT,
            state_changed: true,
        }
    }

    pub fn new() -> Self {
        Self::idling_bottom()
    }

    /// Update animation timers.
    pub fn update_timer(&mut self, delta: Duration) {
        self.timer.tick(delta);
        if !self.timer.finished() {
            return;
        }
        self.frame = (self.frame + 1)
            % match self.state {
                PlayerAnimationState::IdlingB => Self::IDLE_FRAMES,
                PlayerAnimationState::IdlingT => Self::IDLE_FRAMES,
                PlayerAnimationState::IdlingL => Self::IDLE_FRAMES,
                PlayerAnimationState::IdlingR => Self::IDLE_FRAMES,
                PlayerAnimationState::WalkingT => Self::WALKING_FRAMES,
                PlayerAnimationState::WalkingL => Self::WALKING_FRAMES,
                PlayerAnimationState::WalkingR => Self::WALKING_FRAMES,
                PlayerAnimationState::WalkingB => Self::WALKING_FRAMES,
            };
    }

    /// Update animation state if it changes.
    pub fn update_state(&mut self, state: PlayerAnimationState) {
        if self.state != state {
            match state {
                PlayerAnimationState::IdlingB => *self = Self::idling_bottom(),
                PlayerAnimationState::WalkingB => *self = Self::walking_bottom(),
                PlayerAnimationState::IdlingT => *self = Self::idling_top(),
                PlayerAnimationState::IdlingL => *self = Self::idling_left(),
                PlayerAnimationState::IdlingR => *self = Self::idling_right(),
                PlayerAnimationState::WalkingT => *self = Self::walking_top(),
                PlayerAnimationState::WalkingL => *self = Self::walking_left(),
                PlayerAnimationState::WalkingR => *self = Self::walking_right(),
            }
        }
    }

    /// Whether animation changed this tick.
    pub fn changed(&mut self) -> bool {
        if self.state_changed {
            true
        } else {
            self.timer.finished()
        }
    }
    pub fn set_state_changed(&mut self, state_changed: bool) {
        self.state_changed = state_changed;
    }
    /// Return sprite index in the atlas.
    pub fn get_atlas_index(&self) -> usize {
        match self.state {
            // Bottom facing animations (first row of the sprite sheet)
            PlayerAnimationState::IdlingB => self.frame,
            PlayerAnimationState::WalkingB => 2 + self.frame,

            // Left facing animations (second row of the sprite sheet)
            PlayerAnimationState::IdlingL => 8 + self.frame,
            PlayerAnimationState::WalkingL => 10 + self.frame,

            // Right facing animations (third row of the sprite sheet)
            PlayerAnimationState::IdlingR => 12 + self.frame,
            PlayerAnimationState::WalkingR => 14 + self.frame,

            // Top facing animations (fourth row of the sprite sheet)
            PlayerAnimationState::IdlingT => 4 + self.frame,
            PlayerAnimationState::WalkingT => 6 + self.frame,
        }
    }
}
