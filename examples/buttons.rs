use std::time::Duration;

use bevy::{
    prelude::{
        info, App, BuildChildren, Color, Commands, Component, Entity, EventReader, Query, Startup,
        Update, With,
    },
    DefaultPlugins,
};

use epui::{
    event::ClickEvent,
    prelude::*,
    property::{
        collision::BoxCollisionBundle,
        state::{click::ClickEffectTransition, hover::HoverEffectTransition},
    },
};

fn main() {
    App::new()
        .add_systems(Startup, setup)
        .add_plugins(DefaultPlugins)
        .add_plugins(UiPlugin)
        .add_systems(Update, on_button_click)
        .run();
}

#[derive(Component)]
struct Button;

fn setup(mut commands: Commands) {
    commands.spawn(UiCameraBundle::default());

    commands
        .spawn((
            UiBoxBundle {
                position: Position::new(50, 50),
                size: Size::new(200, 60),

                color: ColoredElement::new(Color::GRAY),

                ..Default::default()
            },
            CornersRoundness::from_scalar(1.0f32),
            ClickEffectTransition::new(
                ColoredElement::new(Color::DARK_GREEN),
                Duration::from_millis(200),
                Duration::from_millis(400),
            ),
            HoverEffectTransition::new(
                ColoredElement::new(Color::DARK_GRAY),
                Duration::from_millis(100),
                Duration::from_millis(200),
            ),
            BoxCollisionBundle::new(),
            Button,
        ))
        .with_children(|parent| {
            parent.spawn((
                UiTextBundle {
                    text: UiText(String::from("Button A")),
                    font_size: FontSize(32),

                    position: Position::new(60, 60),
                    size: Size::new(180, 40),

                    ..Default::default()
                },
                BoxCollisionBundle::new(),
            ));
        });

    commands
        .spawn((
            UiBoxBundle {
                position: Position::new(50, 150),
                size: Size::new(200, 60),
                color: ColoredElement::new(Color::GRAY),

                ..Default::default()
            },
            ClickEffectTransition::new(
                ColoredElement::new(Color::DARK_GREEN),
                Duration::from_millis(200),
                Duration::from_millis(400),
            ),
            HoverEffectTransition::new(
                ColoredElement::new(Color::DARK_GRAY),
                Duration::from_millis(100),
                Duration::from_millis(200),
            ),
            AABBCollisionBundle::new(),
            Button,
        ))
        .with_children(|parent| {
            parent.spawn((
                UiTextBundle {
                    text: UiText(String::from("Button B")),
                    font_size: FontSize(32),

                    position: Position::new(60, 160),
                    size: Size::new(180, 40),

                    ..Default::default()
                },
                BoxCollisionBundle::new(),
            ));
        });
}

fn on_button_click(
    buttons: Query<Entity, With<Button>>,
    mut click_events: EventReader<ClickEvent>,
) {
    for click_event in click_events.iter() {
        let Ok(_) = buttons.get(click_event.element) else {
            continue;
        };

        info!("Button got clicked !");
    }
}
