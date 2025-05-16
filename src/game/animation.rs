use bevy::{prelude::*, sprite::Anchor};
use rand::prelude::*;
use std::time::Duration;

use crate::{
    AppSystems,
    audio::sound_effect,
    game::{movement::MovementController, player::PlayerAssets},
};

pub(super) fn plugin(app: &mut App) {
    // Animate and play sound effects based on controls.
    app.register_type::<PlayerAnimation>();
    app.add_systems(
        Update,
        (
            update_animation_timer.in_set(AppSystems::TickTimers),
            (
                update_player_actions,
                update_animation_movement,
                update_animation_atlas,
                trigger_step_sound_effect,
            )
                .chain()
                .run_if(resource_exists::<PlayerAssets>)
                .in_set(AppSystems::Update),
        ),
    );
}

/// Represents the direction of the player animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Component, Debug, Default)]
pub struct PlayerActionState {
    pub current_action: Option<ActionType>,
    pub action_progress: f32, // 0.0 to 1.0
}

/// Represents the action type of the player animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    Hoeing,
    Watering,
    Chopping,
}

impl PlayerAnimationState {
    // Get the direction component of this state
    pub fn get_direction(&self) -> Direction {
        match self {
            Self::IdlingT | Self::WalkingT | Self::HoeingT | Self::WateringT | Self::ChoppingT => {
                Direction::Top
            }
            Self::IdlingB | Self::WalkingB | Self::HoeingB | Self::WateringB | Self::ChoppingB => {
                Direction::Bottom
            }
            Self::IdlingL | Self::WalkingL | Self::HoeingL | Self::WateringL | Self::ChoppingL => {
                Direction::Left
            }
            Self::IdlingR | Self::WalkingR | Self::HoeingR | Self::WateringR | Self::ChoppingR => {
                Direction::Right
            }
        }
    }

    // Create a state from action and direction
    pub fn from_action_and_direction(action: ActionType, direction: Direction) -> Self {
        match (action, direction) {
            (ActionType::Hoeing, Direction::Top) => Self::HoeingT,
            (ActionType::Hoeing, Direction::Bottom) => Self::HoeingB,
            (ActionType::Hoeing, Direction::Left) => Self::HoeingL,
            (ActionType::Hoeing, Direction::Right) => Self::HoeingR,

            (ActionType::Watering, Direction::Top) => Self::WateringT,
            (ActionType::Watering, Direction::Bottom) => Self::WateringB,
            (ActionType::Watering, Direction::Left) => Self::WateringL,
            (ActionType::Watering, Direction::Right) => Self::WateringR,

            (ActionType::Chopping, Direction::Top) => Self::ChoppingT,
            (ActionType::Chopping, Direction::Bottom) => Self::ChoppingB,
            (ActionType::Chopping, Direction::Left) => Self::ChoppingL,
            (ActionType::Chopping, Direction::Right) => Self::ChoppingR,
        }
    }
}

fn update_player_actions(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(
        &mut PlayerAnimation,
        &mut PlayerActionState,
        &MovementController,
    )>,
) {
    for (mut animation, mut action_state, controller) in &mut player_query {
        // Get current direction from animation state
        let direction = animation.state.get_direction();

        // Check for new action triggers
        if action_state.current_action.is_none() {
            // Only allow starting actions when not moving
            if controller.intent == Vec2::ZERO {
                if input.just_pressed(KeyCode::KeyE) {
                    // Start watering action
                    action_state.current_action = Some(ActionType::Watering);
                    action_state.action_progress = 0.0;

                    // Set animation state based on current direction
                    let new_state = PlayerAnimationState::from_action_and_direction(
                        action_state.current_action.unwrap(),
                        direction,
                    );
                    animation.update_state(new_state);
                } else if input.just_pressed(KeyCode::KeyQ) {
                    // Start chopping action
                    action_state.current_action = Some(ActionType::Hoeing);
                    action_state.action_progress = 0.0;

                    // Set animation state based on current direction
                    let new_state = PlayerAnimationState::from_action_and_direction(
                        action_state.current_action.unwrap(),
                        direction,
                    );
                    animation.update_state(new_state);
                } else if input.just_pressed(KeyCode::KeyF) {
                    // Start hoeing action
                    action_state.current_action = Some(ActionType::Chopping);
                    action_state.action_progress = 0.0;

                    // Set animation state based on current direction
                    let new_state = PlayerAnimationState::from_action_and_direction(
                        action_state.current_action.unwrap(),
                        direction,
                    );
                    animation.update_state(new_state);
                }
            }
        } else {
            // Update existing action
            action_state.action_progress += time.delta_secs();

            // Check if action is complete (adjust times based on your animations)
            let action_duration = match action_state.current_action {
                Some(ActionType::Watering) => 0.6, // seconds
                Some(ActionType::Hoeing) => 0.6,   // seconds
                Some(ActionType::Chopping) => 0.6, // seconds
                _ => 0.0,
            };

            if action_state.action_progress >= action_duration {
                // Action complete, return to idle state
                action_state.current_action = None;

                // Return to idle state based on current direction
                let new_state = match direction {
                    Direction::Top => PlayerAnimationState::IdlingT,
                    Direction::Bottom => PlayerAnimationState::IdlingB,
                    Direction::Left => PlayerAnimationState::IdlingL,
                    Direction::Right => PlayerAnimationState::IdlingR,
                };
                animation.update_state(new_state);
            }
        }
    }
}

/// Update the sprite direction and animation state (idling/walking).
fn update_animation_movement(
    mut player_query: Query<(
        &MovementController,
        &mut PlayerAnimation,
        &PlayerActionState,
    )>,
) {
    for (controller, mut animation, state) in &mut player_query {
        // If the player is performing an action, skip movement animation
        if state.current_action.is_some() {
            continue;
        }
        let intent = controller.intent;
        let current_direction = animation.state.get_direction();

        // Determine new direction and action based on movement
        let animation_state = if intent == Vec2::ZERO {
            match current_direction {
                Direction::Top => PlayerAnimationState::IdlingT,
                Direction::Bottom => PlayerAnimationState::IdlingB,
                Direction::Left => PlayerAnimationState::IdlingL,
                Direction::Right => PlayerAnimationState::IdlingR,
            }
        } else {
            // Determine direction from movement and set action to walking
            if intent.y.abs() > intent.x.abs() {
                if intent.y > 0.0 {
                    PlayerAnimationState::WalkingT
                } else {
                    PlayerAnimationState::WalkingB
                }
            } else if intent.x > 0.0 {
                PlayerAnimationState::WalkingR
            } else {
                PlayerAnimationState::WalkingL
            }
        };

        if animation.state != animation_state {
            animation.set_state_changed(true);
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
fn update_animation_atlas(mut query: Query<(&PlayerAnimation, &mut Sprite)>) {
    for (animation, mut sprite) in &mut query {
        let Some(atlas) = sprite.texture_atlas.as_mut() else {
            continue;
        };
        if animation.changed() {
            atlas.index = animation.get_atlas_index();
            sprite.anchor = Anchor::Custom(animation.state.get_anchor_point(animation.frame));
        }
    }
}

/// If the player is moving, play a step sound effect synchronized with the
/// animation.
fn trigger_step_sound_effect(
    mut commands: Commands,
    player_assets: Res<PlayerAssets>,
    mut step_query: Query<&mut PlayerAnimation>,
) {
    for mut animation in &mut step_query {
        if animation.state.is_walking() && animation.changed() {
            let rng = &mut rand::thread_rng();
            let random_step = player_assets
                .steps
                .choose(rng)
                .expect("Player assets should exist!")
                .clone();
            commands.spawn(sound_effect(random_step));
        }
        animation.set_state_changed(false);
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
    HoeingT,
    HoeingB,
    HoeingL,
    HoeingR,
    WateringT,
    WateringB,
    WateringL,
    WateringR,
    ChoppingT,
    ChoppingB,
    ChoppingL,
    ChoppingR,
}

impl PlayerAnimationState {
    fn is_walking(&self) -> bool {
        matches!(
            self,
            PlayerAnimationState::WalkingT
                | PlayerAnimationState::WalkingB
                | PlayerAnimationState::WalkingL
                | PlayerAnimationState::WalkingR
        )
    }

    fn _is_idling(&self) -> bool {
        matches!(
            self,
            PlayerAnimationState::IdlingT
                | PlayerAnimationState::IdlingB
                | PlayerAnimationState::IdlingL
                | PlayerAnimationState::IdlingR
        )
    }
    pub fn get_anchor_point(&self, frame: usize) -> Vec2 {
        match self {
            PlayerAnimationState::HoeingL => {
                if frame == 1 {
                    Vec2::new(0.2, 0.0)
                } else {
                    Vec2::ZERO
                }
            }
            PlayerAnimationState::HoeingR => {
                if frame == 1 {
                    Vec2::new(-0.2, 0.0)
                } else {
                    Vec2::ZERO
                }
            }
            PlayerAnimationState::WateringR => {
                if frame == 1 {
                    Vec2::new(-0.2, 0.0)
                } else {
                    Vec2::new(-0.25, 0.0)
                }
            }
            PlayerAnimationState::WateringL => {
                if frame == 1 {
                    Vec2::new(0.3, 0.0)
                } else {
                    Vec2::new(0.25, 0.0)
                }
            }
            PlayerAnimationState::ChoppingR => {
                if frame == 1 {
                    Vec2::new(-0.2, 0.0)
                } else {
                    Vec2::new(0.2, 0.0)
                }
            }
            PlayerAnimationState::ChoppingL => {
                if frame == 1 {
                    Vec2::new(0.2, 0.0)
                } else {
                    Vec2::new(-0.2, 0.0)
                }
            }
            PlayerAnimationState::ChoppingT => {
                if frame == 1 {
                    Vec2::new(-0.1, 0.0)
                } else {
                    Vec2::ZERO
                }
            }
            PlayerAnimationState::ChoppingB => {
                if frame == 1 {
                    Vec2::ZERO
                } else {
                    Vec2::new(0.1, 0.0)
                }
            }
            _ => Vec2::ZERO,
        }
    }
}

impl PlayerAnimation {
    const IDLE_INTERVAL: Duration = Duration::from_millis(500);
    const WALKING_INTERVAL: Duration = Duration::from_millis(150);
    const HOEING_INTERVAL: Duration = Duration::from_millis(300);
    const WATERING_INTERVAL: Duration = Duration::from_millis(300);
    const CHOPPING_INTERVAL: Duration = Duration::from_millis(300);
    const WALKING_FRAMES: usize = 2;
    const HOEING_FRAMES: usize = 2;
    const WATERING_FRAMES: usize = 2;
    const IDLE_FRAMES: usize = 2;
    const CHOPPING_FRAMES: usize = 2;

    fn internal_new(duration: Duration, state: PlayerAnimationState) -> Self {
        Self {
            timer: Timer::new(duration, TimerMode::Repeating),
            frame: 0,
            state,
            state_changed: true,
        }
    }

    pub fn new() -> Self {
        Self::internal_new(Self::IDLE_INTERVAL, PlayerAnimationState::IdlingB)
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
                PlayerAnimationState::HoeingT => Self::HOEING_FRAMES,
                PlayerAnimationState::HoeingL => Self::HOEING_FRAMES,
                PlayerAnimationState::HoeingR => Self::HOEING_FRAMES,
                PlayerAnimationState::HoeingB => Self::HOEING_FRAMES,
                PlayerAnimationState::WateringT => Self::WATERING_FRAMES,
                PlayerAnimationState::WateringL => Self::WATERING_FRAMES,
                PlayerAnimationState::WateringR => Self::WATERING_FRAMES,
                PlayerAnimationState::WateringB => Self::WATERING_FRAMES,
                PlayerAnimationState::ChoppingT => Self::CHOPPING_FRAMES,
                PlayerAnimationState::ChoppingB => Self::CHOPPING_FRAMES,
                PlayerAnimationState::ChoppingL => Self::CHOPPING_FRAMES,
                PlayerAnimationState::ChoppingR => Self::CHOPPING_FRAMES,
            };
    }

    /// Update animation state if it changes.
    pub fn update_state(&mut self, state: PlayerAnimationState) {
        if self.state != state {
            match state {
                PlayerAnimationState::IdlingB => {
                    *self = Self::internal_new(Self::IDLE_INTERVAL, PlayerAnimationState::IdlingB)
                }
                PlayerAnimationState::IdlingT => {
                    *self = Self::internal_new(Self::IDLE_INTERVAL, PlayerAnimationState::IdlingT)
                }
                PlayerAnimationState::IdlingL => {
                    *self = Self::internal_new(Self::IDLE_INTERVAL, PlayerAnimationState::IdlingL)
                }
                PlayerAnimationState::IdlingR => {
                    *self = Self::internal_new(Self::IDLE_INTERVAL, PlayerAnimationState::IdlingR)
                }
                PlayerAnimationState::WalkingB => {
                    *self =
                        Self::internal_new(Self::WALKING_INTERVAL, PlayerAnimationState::WalkingB)
                }
                PlayerAnimationState::WalkingT => {
                    *self =
                        Self::internal_new(Self::WALKING_INTERVAL, PlayerAnimationState::WalkingT)
                }
                PlayerAnimationState::WalkingL => {
                    *self =
                        Self::internal_new(Self::WALKING_INTERVAL, PlayerAnimationState::WalkingL)
                }
                PlayerAnimationState::WalkingR => {
                    *self =
                        Self::internal_new(Self::WALKING_INTERVAL, PlayerAnimationState::WalkingR)
                }
                PlayerAnimationState::HoeingT => {
                    *self = Self::internal_new(Self::HOEING_INTERVAL, PlayerAnimationState::HoeingT)
                }
                PlayerAnimationState::HoeingB => {
                    *self = Self::internal_new(Self::HOEING_INTERVAL, PlayerAnimationState::HoeingB)
                }
                PlayerAnimationState::HoeingL => {
                    *self = Self::internal_new(Self::HOEING_INTERVAL, PlayerAnimationState::HoeingL)
                }
                PlayerAnimationState::HoeingR => {
                    *self = Self::internal_new(Self::HOEING_INTERVAL, PlayerAnimationState::HoeingR)
                }
                PlayerAnimationState::WateringT => {
                    *self =
                        Self::internal_new(Self::WATERING_INTERVAL, PlayerAnimationState::WateringT)
                }
                PlayerAnimationState::WateringB => {
                    *self =
                        Self::internal_new(Self::WATERING_INTERVAL, PlayerAnimationState::WateringB)
                }
                PlayerAnimationState::WateringL => {
                    *self =
                        Self::internal_new(Self::WATERING_INTERVAL, PlayerAnimationState::WateringL)
                }
                PlayerAnimationState::WateringR => {
                    *self =
                        Self::internal_new(Self::WATERING_INTERVAL, PlayerAnimationState::WateringR)
                }
                PlayerAnimationState::ChoppingT => {
                    *self =
                        Self::internal_new(Self::CHOPPING_INTERVAL, PlayerAnimationState::ChoppingT)
                }
                PlayerAnimationState::ChoppingB => {
                    *self =
                        Self::internal_new(Self::CHOPPING_INTERVAL, PlayerAnimationState::ChoppingB)
                }
                PlayerAnimationState::ChoppingL => {
                    *self =
                        Self::internal_new(Self::CHOPPING_INTERVAL, PlayerAnimationState::ChoppingL)
                }
                PlayerAnimationState::ChoppingR => {
                    *self =
                        Self::internal_new(Self::CHOPPING_INTERVAL, PlayerAnimationState::ChoppingR)
                }
            }
        }
    }

    /// Whether animation changed this tick.
    pub fn changed(&self) -> bool {
        if self.state_changed {
            true
        } else {
            self.timer.finished()
        }
    }

    /// Set animation state changed.
    pub fn set_state_changed(&mut self, state_changed: bool) {
        self.state_changed = state_changed;
    }

    /// Return sprite index in the atlas.
    pub fn get_atlas_index(&self) -> usize {
        match self.state {
            PlayerAnimationState::IdlingB => self.frame,
            PlayerAnimationState::WalkingB => 2 + self.frame,
            PlayerAnimationState::IdlingL => 32 + self.frame,
            PlayerAnimationState::WalkingL => 34 + self.frame,
            PlayerAnimationState::IdlingR => 48 + self.frame,
            PlayerAnimationState::WalkingR => 50 + self.frame,
            PlayerAnimationState::IdlingT => 16 + self.frame,
            PlayerAnimationState::WalkingT => 18 + self.frame,
            PlayerAnimationState::HoeingT => 20 + self.frame,
            PlayerAnimationState::HoeingB => 4 + self.frame,
            PlayerAnimationState::HoeingL => 36 + self.frame,
            PlayerAnimationState::HoeingR => 52 + self.frame,
            PlayerAnimationState::WateringT => 24 + self.frame,
            PlayerAnimationState::WateringB => 8 + self.frame,
            PlayerAnimationState::WateringL => 40 + self.frame,
            PlayerAnimationState::WateringR => 56 + self.frame,
            PlayerAnimationState::ChoppingT => 22 + self.frame,
            PlayerAnimationState::ChoppingB => 6 + self.frame,
            PlayerAnimationState::ChoppingL => 38 + self.frame,
            PlayerAnimationState::ChoppingR => 54 + self.frame,
        }
    }
}
