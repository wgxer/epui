use std::time::Duration;

use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::{
        App, BuildChildren, Color, Commands, Component, Entity, EventReader, IntoSystemAppConfig,
        MouseButton, Query, Rect, Vec2, With,
    },
    window::{PrimaryWindow, Window},
    DefaultPlugins,
};

use epui::{
    prelude::*,
    property::{
        collision::{BoxCollisionBundle, Collision},
        VisibleRegion,
    },
};

fn main() {
    App::new()
        .add_system(setup.on_startup())
        .add_plugins(DefaultPlugins)
        .add_plugin(UiPlugin)
        .add_system(on_mouse_click)
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

fn on_mouse_click(
    mut commands: Commands,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    elements: Query<(Entity, &Position, &Size, &Collision, Option<&VisibleRegion>)>,
    mut events: EventReader<MouseButtonInput>,
) {
    let primary_window = primary_window.single();

    let Some(cursor_position) = primary_window.cursor_position() else {
        return;
    };

    let cursor_position = Vec2::new(
        cursor_position.x,
        primary_window.height() - cursor_position.y,
    )
    .round();

    for click_event in events.into_iter() {
        if click_event.button == MouseButton::Left {
            for (entity, position, size, collision, visible_region) in elements.iter() {
                let visible_region = match visible_region {
                    Some(visible_region) => visible_region.clone(),
                    None => VisibleRegion::new(position.x, position.y, size.width, size.height),
                };

                if Rect::from(visible_region).contains(cursor_position) {
                    if collision
                        .0
                        .contains(position.clone(), size.clone(), cursor_position)
                    {
                        commands.entity(entity).insert(Transition::new(
                            ColoredElement::new(match click_event.state {
                                ButtonState::Pressed => Color::DARK_GREEN,
                                ButtonState::Released => Color::GRAY,
                            }),
                            Duration::from_millis(100),
                        ));
                    }
                }
            }
        }
    }
}
