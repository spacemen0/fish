//! The settings screen accessible from the title screen.
//! We can add all manner of settings and accessibility options here.
//! For 3D, we'd also place the camera sensitivity and FOV here.

use bevy::{audio::Volume, prelude::*};

use crate::{
    states::{GameState, PreviousState},
    theme::prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Settings), spawn_settings_screen);

    app.register_type::<GlobalVolumeLabel>();
    app.add_systems(
        Update,
        (update_volume_label, update_game_speed_label).run_if(in_state(GameState::Settings)),
    );
}

fn spawn_settings_screen(mut commands: Commands) {
    commands
        .spawn_scene(bsn! {
            widget::ui_root("Settings Screen")
            Children [
                widget::header("Settings"),
                (
                    Name("Settings Grid")
                    Node {
                        display: Display::Grid,
                        row_gap: px(10.0),
                        column_gap: px(30.0),
                        grid_template_columns: { RepeatedGridTrack::px::<RepeatedGridTrack>(2, 400.0) },
                    }
                    Children [
                        (
                            widget::label("Audio Volume")
                            Node {
                                justify_self: JustifySelf::End,
                            }
                        ),
                        volume_widget(),
                        (
                            widget::label("Game Speed")
                            Node {
                                justify_self: JustifySelf::End,
                            }
                        ),
                        game_speed_widget(),
                    ]
                ),
                widget::button("Back", enter_last_screen),
            ]
        })
        .insert(DespawnOnExit(GameState::Settings));
}

fn volume_widget() -> impl Scene {
    bsn! {
        Node {
            justify_self: JustifySelf::Start,
        }
        Children [
            widget::button_small("-", lower_volume),
            (
                Node {
                    padding: { UiRect::horizontal(px(10.0)) },
                    justify_content: JustifyContent::Center,
                }
                Children [
                    (
                        widget::label("")
                        GlobalVolumeLabel
                    )
                ]
            ),
            widget::button_small("+", raise_volume),
        ]
    }
}

fn game_speed_widget() -> impl Scene {
    bsn! {
        Node {
            justify_self: JustifySelf::Start,
        }
        Children [
            widget::button_small("-", lower_game_speed),
            (
                Node {
                    padding: { UiRect::horizontal(px(10.0)) },
                    justify_content: JustifyContent::Center,
                }
                Children [
                    (
                        widget::label("")
                        GlobalGameSpeedLabel
                    )
                ]
            ),
            widget::button_small("+", raise_game_speed),
        ]
    }
}

const MIN_VOLUME: f32 = 0.0;
const MAX_VOLUME: f32 = 3.0;
const MAX_GAME_SPEED: f32 = 3.0;
const MIN_GAME_SPEED: f32 = 0.2;

fn lower_volume(_: On<Pointer<Click>>, mut global_volume: ResMut<GlobalVolume>) {
    let new_factor = global_volume.volume.to_linear() - 0.1;
    global_volume.volume = Volume::Linear(new_factor.max(MIN_VOLUME));
}

fn raise_volume(_: On<Pointer<Click>>, mut global_volume: ResMut<GlobalVolume>) {
    let new_factor = global_volume.volume.to_linear() + 0.1;
    global_volume.volume = Volume::Linear(new_factor.min(MAX_VOLUME));
}

fn lower_game_speed(_: On<Pointer<Click>>, mut time: ResMut<Time<Virtual>>) {
    let new_speed = time.relative_speed() - 0.1;
    time.set_relative_speed(new_speed.max(MIN_GAME_SPEED));
}

fn raise_game_speed(_: On<Pointer<Click>>, mut time: ResMut<Time<Virtual>>) {
    let new_speed = time.relative_speed() + 0.1;
    time.set_relative_speed(new_speed.min(MAX_GAME_SPEED));
}
#[derive(Component, Reflect, Clone, Copy, Default)]
#[reflect(Component)]
struct GlobalVolumeLabel;

#[derive(Component, Reflect, Clone, Copy, Default)]
#[reflect(Component)]
struct GlobalGameSpeedLabel;

fn update_volume_label(
    mut label: Single<&mut Text, With<GlobalVolumeLabel>>,
    global_volume: Res<GlobalVolume>,
) {
    let factor = global_volume.volume.to_linear();
    let percent = (factor * 100.0).round();
    let text = format!("{percent}%");
    label.0 = text;
}

fn update_game_speed_label(
    mut label: Single<&mut Text, With<GlobalGameSpeedLabel>>,
    time: Res<Time<Virtual>>,
) {
    let speed = time.relative_speed();
    let text = format!("{speed:.1}x");
    label.0 = text;
}

fn enter_last_screen(
    _: On<Pointer<Click>>,
    mut next_screen: ResMut<NextState<GameState>>,
    mut previous_state: ResMut<PreviousState>,
) {
    next_screen.set(previous_state.0.clone());
    previous_state.0 = GameState::Settings;
}
