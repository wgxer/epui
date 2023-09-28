use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::{
        warn, Commands, Component, Entity, EventReader, EventWriter, MouseButton, Plugin, Query,
        Rect, Vec2, With,
    },
    window::{PrimaryWindow, Window},
};

use crate::{
    prelude::{Position, Size},
    property::{collision::Collision, VisibleRegion},
};

pub struct UiEventPlugin;

impl Plugin for UiEventPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<PressEvent>()
            .add_event::<ReleaseEvent>()
            .add_event::<ClickEvent>()
            .add_system(on_mouse_click_start)
            .add_system(on_mouse_click_end);
    }
}

pub struct PressEvent {
    pub element: Entity,
}

pub struct ReleaseEvent {
    pub element: Entity,
}

pub struct ClickEvent {
    pub element: Entity,
}

#[derive(Component)]
pub struct ElementPressed;

fn on_mouse_click_start(
    mut commands: Commands,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    elements: Query<(Entity, &Position, &Size, &Collision, Option<&VisibleRegion>)>,
    mut mouse_click_events: EventReader<MouseButtonInput>,
    mut press_events: EventWriter<PressEvent>,
) {
    let Ok(primary_window) = primary_window.get_single() else {
        warn!("Couldn't get primary window");

        return;
    };

    let Some(cursor_position) = primary_window.cursor_position() else {
        return;
    };

    let cursor_position = Vec2::new(
        cursor_position.x,
        primary_window.height() - cursor_position.y,
    )
    .round();

    for mouse_click_event in mouse_click_events.iter() {
        if mouse_click_event.button == MouseButton::Left {
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
                        if mouse_click_event.button == MouseButton::Left
                            && mouse_click_event.state == ButtonState::Pressed
                        {
                            commands.entity(entity).insert(ElementPressed);
                            press_events.send(PressEvent { element: entity });
                        }
                    }
                }
            }
        }
    }
}

fn on_mouse_click_end(
    mut commands: Commands,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    elements: Query<
        (Entity, &Position, &Size, &Collision, Option<&VisibleRegion>),
        With<ElementPressed>,
    >,
    mut mouse_click_events: EventReader<MouseButtonInput>,
    mut release_events: EventWriter<ReleaseEvent>,
    mut click_events: EventWriter<ClickEvent>,
) {
    let Ok(primary_window) = primary_window.get_single() else {
        warn!("Couldn't get primary window");

        return;
    };

    let Some(cursor_position) = primary_window.cursor_position() else {
        return;
    };

    let cursor_position = Vec2::new(
        cursor_position.x,
        primary_window.height() - cursor_position.y,
    )
    .round();

    for mouse_click_event in mouse_click_events.iter() {
        if mouse_click_event.button == MouseButton::Left {
            for (entity, position, size, collision, visible_region) in elements.iter() {
                if mouse_click_event.button == MouseButton::Left
                    && mouse_click_event.state == ButtonState::Released
                {
                    commands.entity(entity).remove::<ElementPressed>();
                    release_events.send(ReleaseEvent { element: entity });

                    let visible_region = match visible_region {
                        Some(visible_region) => visible_region.clone(),
                        None => VisibleRegion::new(position.x, position.y, size.width, size.height),
                    };

                    if Rect::from(visible_region).contains(cursor_position) {
                        if collision
                            .0
                            .contains(position.clone(), size.clone(), cursor_position)
                        {
                            click_events.send(ClickEvent { element: entity });
                        }
                    }
                }
            }
        }
    }
}
