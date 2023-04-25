use std::time::Duration;

use bevy::{
    prelude::{
        info, App, BuildChildren, Color, Commands, Component, Entity, EventReader,
        IntoSystemAppConfig, Query, With,
    },
    DefaultPlugins,
};

use epui::{
    event::{ClickEvent, PressEvent, ReleaseEvent},
    prelude::*,
    property::collision::BoxCollisionBundle,
};

fn main() {
    App::new()
        .add_system(setup.on_startup())
        .add_plugins(DefaultPlugins)
        .add_plugin(UiPlugin)
        .add_system(update_button_color)
        .add_system(on_button_click)
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
            BoxCollisionBundle::new(),
            Button,
        ))
        .with_children(|parent| {
            parent.spawn(UiTextBundle {
                text: UiText {
                    text: String::from("Button A"),
                    font_size: 36,
                },

                position: Position::new(60, 60),
                size: Size::new(180, 40),

                ..Default::default()
            });
        });

    commands
        .spawn((
            UiBoxBundle {
                position: Position::new(50, 150),
                size: Size::new(200, 60),
                color: ColoredElement::new(Color::GRAY),

                ..Default::default()
            },
            AABBCollisionBundle::new(),
            Button,
        ))
        .with_children(|parent| {
            parent.spawn(UiTextBundle {
                text: UiText {
                    text: String::from("Button B"),
                    font_size: 36,
                },

                position: Position::new(60, 160),
                size: Size::new(180, 40),

                ..Default::default()
            });
        });
}

fn update_button_color(
    mut commands: Commands,
    buttons: Query<Entity, With<Button>>,
    mut press_events: EventReader<PressEvent>,
    mut release_events: EventReader<ReleaseEvent>,
) {
    for press_event in press_events.iter() {
        let Ok(button_entity) = buttons.get(press_event.element) else {
            continue;
        };

        commands.entity(button_entity).insert(Transition::new(
            ColoredElement::new(Color::DARK_GREEN),
            Duration::from_millis(100),
        ));
    }

    for release_event in release_events.iter() {
        let Ok(button_entity) = buttons.get(release_event.element) else {
            continue;
        };

        commands.entity(button_entity).insert(Transition::new(
            ColoredElement::new(Color::GRAY),
            Duration::from_millis(100),
        ));
    }
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
