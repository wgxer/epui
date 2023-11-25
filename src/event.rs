use bevy::{
    input::{
        mouse::{MouseButtonInput, MouseMotion},
        ButtonState,
    },
    log::warn,
    prelude::{
        Commands, Component, Entity, Event, EventReader, EventWriter, MouseButton, Plugin, Query,
        Rect, Update, With, Without,
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
        app.add_event::<HoverEnterEvent>()
            .add_event::<HoverExitEvent>()
            .add_event::<PressEvent>()
            .add_event::<ReleaseEvent>()
            .add_event::<ClickEvent>()
            .add_systems(
                Update,
                (on_mouse_move, on_mouse_click_start, on_mouse_click_end),
            );
    }
}

#[derive(Event)]
pub struct HoverEnterEvent {
    pub element: Entity,
}

#[derive(Event)]
pub struct HoverExitEvent {
    pub element: Entity,
}

#[derive(Component)]
pub struct ElementHovered;

fn on_mouse_move(
    mut commands: Commands,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    elements_not_hovered: Query<
        (Entity, &Position, &Size, &Collision, Option<&VisibleRegion>),
        Without<ElementHovered>,
    >,
    elements_hovered: Query<
        (Entity, &Position, &Size, &Collision, Option<&VisibleRegion>),
        With<ElementHovered>,
    >,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut hover_enter_events: EventWriter<HoverEnterEvent>,
    mut hover_exit_events: EventWriter<HoverExitEvent>,
) {
    let Ok(primary_window) = primary_window.get_single() else {
        warn!("Couldn't get primary window");

        return;
    };

    let Some(cursor_position) = primary_window.cursor_position() else {
        return;
    };

    let cursor_position = cursor_position.round();

    if mouse_motion_events.read().next().is_some() {
        for (entity, position, size, collision, visible_region) in elements_not_hovered.iter() {
            let visible_region = match visible_region {
                Some(visible_region) => visible_region.clone(),
                None => VisibleRegion::new(position.x, position.y, size.width, size.height),
            };

            if Rect::from(visible_region).contains(cursor_position) {
                if collision
                    .0
                    .contains(position.clone(), size.clone(), cursor_position)
                {
                    commands.entity(entity).insert(ElementHovered);
                    hover_enter_events.send(HoverEnterEvent { element: entity });
                }
            }
        }

        for (entity, position, size, collision, visible_region) in elements_hovered.iter() {
            let visible_region = match visible_region {
                Some(visible_region) => visible_region.clone(),
                None => VisibleRegion::new(position.x, position.y, size.width, size.height),
            };

            if Rect::from(visible_region).contains(cursor_position) {
                if collision
                    .0
                    .contains(position.clone(), size.clone(), cursor_position)
                {
                    continue;
                }
            }

            commands.entity(entity).remove::<ElementHovered>();
            hover_exit_events.send(HoverExitEvent { element: entity });
        }
    }
}

#[derive(Event)]
pub struct PressEvent {
    pub element: Entity,
}

#[derive(Event)]
pub struct ReleaseEvent {
    pub element: Entity,
}

#[derive(Event)]
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

    let cursor_position = cursor_position.round();

    for mouse_click_event in mouse_click_events.read() {
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

    let cursor_position = cursor_position.round();

    for mouse_click_event in mouse_click_events.read() {
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
