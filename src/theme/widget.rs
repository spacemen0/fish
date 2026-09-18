//! Helper functions for creating common widgets.

use bevy::{
    ecs::{event::EntityEvent, system::IntoObserverSystem},
    prelude::*,
};

use crate::theme::{interaction::InteractionPalette, palette::*};

/// A root UI node that fills the window and centers its content.
pub fn ui_root(name: impl Into<String>) -> impl Scene {
    let name = name.into();
    bsn! {
        Name(name)
        Node {
            position_type: PositionType::Absolute,
            width: percent(100.0),
            height: percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(20.0),
        }
    }
}

/// A simple header label. Bigger than [`label`].
pub fn header(text: impl Into<String>) -> impl Scene {
    let text = text.into();
    bsn! {
        Name("Header")
        Text(text)
        TextFont {
            font_size: px(40.0),
        }
        TextColor(HEADER_TEXT)
    }
}

/// A simple text label.
pub fn label(text: impl Into<String>) -> impl Scene {
    let text = text.into();
    bsn! {
        Name("Label")
        Text(text)
        TextFont {
            font_size: px(24.0),
        }
        TextColor(LABEL_TEXT)
    }
}

/// A large rounded button with text and an action defined as an [`Observer`].
pub fn button<E, B, M, I>(text: impl Into<String>, action: I) -> impl Scene
where
    E: EntityEvent,
    B: Bundle,
    M: 'static,
    I: IntoObserverSystem<E, B, M> + Clone + Send + Sync + 'static,
{
    button_base(
        text,
        action,
        Node {
            width: px(300.0),
            height: px(80.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::MAX,
            ..default()
        },
    )
}

/// A small square button with text and an action defined as an [`Observer`].
pub fn button_small<E, B, M, I>(text: impl Into<String>, action: I) -> impl Scene
where
    E: EntityEvent,
    B: Bundle,
    M: 'static,
    I: IntoObserverSystem<E, B, M> + Clone + Send + Sync + 'static,
{
    button_base(
        text,
        action,
        Node {
            width: px(30.0),
            height: px(30.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
    )
}

/// A simple button with text and an action defined as an [`Observer`]. The button's layout is provided by `button_node`.
fn button_base<E, B, M, I>(
    text: impl Into<String>,
    action: I,
    button_node: Node,
) -> impl Scene
where
    E: EntityEvent,
    B: Bundle,
    M: 'static,
    I: IntoObserverSystem<E, B, M> + Clone + Send + Sync + 'static,
{
    let text = text.into();
    bsn! {
        Name("Button")
        Node
        Children [
            (
                Name("Button Inner")
                Button
                template_value(button_node)
                BackgroundColor(BUTTON_BACKGROUND)
                InteractionPalette {
                    none: BUTTON_BACKGROUND,
                    hovered: BUTTON_HOVERED_BACKGROUND,
                    pressed: BUTTON_PRESSED_BACKGROUND,
                }
                on(action)
                Children [
                    (
                        Name("Button Text")
                        Text(text)
                        TextFont {
                            font_size: px(40.0),
                        }
                        TextColor(BUTTON_TEXT)
                    )
                ]
            )
        ]
    }
}
