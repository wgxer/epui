use std::time::Duration;

use bevy::{
    prelude::{
        info, App, BuildChildren, Color, Commands, Component, Entity, EventReader,
        IntoSystemAppConfig, Query, With, World,
    },
    DefaultPlugins,
};

use epui::{
    event::{ClickEvent, PressEvent, ReleaseEvent},
    prelude::*,
    property::{
        collision::BoxCollisionBundle,
        state::{Active, ActiveOptionExt, AppComponentStateExt, ComponentState},
        transition::transition_system,
    },
};

fn main() {
    App::new()
        .add_component_state::<ClickState, ColoredElement>(())
        .add_component_state::<ClickState, UiText>(())
        .add_system(transition_system::<Clicked<ColoredElement>>)
        .add_system(transition_system::<Clicked<UiText>>)
        .add_system(setup.on_startup())
        .add_plugins(DefaultPlugins)
        .add_plugin(UiPlugin)
        .add_system(update_button_color)
        .add_system(on_button_click)
        .run();
}

#[derive(Component)]
struct Button;

#[derive(Clone)]
struct ClickState;

type Clicked<T> = ComponentState<ClickState, T>;

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
            parent.spawn((
                UiTextBundle {
                    text: UiText {
                        text: String::from("Button A"),
                        font_size: 36,
                    },

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
            AABBCollisionBundle::new(),
            Button,
        ))
        .with_children(|parent| {
            parent.spawn((
                UiTextBundle {
                    text: UiText {
                        text: String::from("Button B"),
                        font_size: 36,
                    },

                    position: Position::new(60, 160),
                    size: Size::new(180, 40),

                    ..Default::default()
                },
                BoxCollisionBundle::new(),
            ));
        });
}

fn update_button_color(
    world: &World,
    mut commands: Commands,
    mut buttons: Query<(Entity, &ColoredElement, Option<&Active<ColoredElement>>), With<Button>>,
    mut texts: Query<(Entity, &UiText, Option<&Active<UiText>>)>,
    mut press_events: EventReader<PressEvent>,
    mut release_events: EventReader<ReleaseEvent>,
) {
    for press_event in press_events.iter() {
        let entity_ref = world.entity(press_event.element);

        let Ok((button_entity, base_color, active_color)) = buttons.get(press_event.element) else {
            let Ok((text_entity, base_text, active_text)) = texts.get(press_event.element) else {
                continue;
            };

            commands.entity(text_entity).insert((
                Clicked::new(active_text.active_or_base(&entity_ref, base_text).clone()),
                Transition::new(
                    Clicked::new(UiText {
                        text: String::from("Clicked"),
                        font_size: 24,
                    }),
                    Duration::from_millis(100),
                ),
            ));

            continue;
        };

        commands.entity(button_entity).insert((
            Clicked::new(active_color.active_or_base(&entity_ref, base_color).clone()),
            Transition::new(
                Clicked::new(ColoredElement::new(Color::DARK_GREEN)),
                Duration::from_millis(100),
            ),
        ));
    }

    for release_event in release_events.iter() {
        let Ok((button_entity, _, active_color)) = buttons.get_mut(release_event.element) else {
            let Ok((text_entity, _, active_text)) = texts.get_mut(release_event.element) else {
                continue;
            };

            if let Some(active_text) = active_text {
                if active_text.is_active_state::<Clicked<UiText>>(world.components()) {
                    commands.entity(text_entity).insert(Transition::new(
                        Clicked::new(
                            active_text
                                .get_state(&world.entity(text_entity), active_text.states_len() - 2)
                                .expect("Couldn't get new active color")
                                .clone(),
                        ),
                        Duration::from_millis(200),
                    ));

                    continue;
                }
            }

            commands.entity(text_entity).remove::<Clicked<UiText>>();

            continue;
        };

        if let Some(active_color) = active_color {
            if active_color.is_active_state::<Clicked<ColoredElement>>(world.components()) {
                commands.entity(button_entity).insert(Transition::new(
                    Clicked::new(
                        active_color
                            .get_state(&world.entity(button_entity), active_color.states_len() - 2)
                            .expect("Couldn't get new active color")
                            .clone(),
                    ),
                    Duration::from_millis(200),
                ));

                continue;
            }
        }

        commands
            .entity(button_entity)
            .remove::<Clicked<ColoredElement>>();
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
