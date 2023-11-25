use std::time::Duration;

use bevy::prelude::{Commands, Component, EventReader, Plugin, Query, Update, With, World};

use crate::{
    event::{HoverEnterEvent, HoverExitEvent},
    prelude::*,
    property::{
        auto_remove::{remove_system, AutoRemove},
        transition::{transition_system, PropertyTransition},
    },
};

use super::{Active, ActiveOptionExt, AppComponentStateExt, ComponentState};

pub struct UiHoverStatePlugin;

impl Plugin for UiHoverStatePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_component_state::<HoverState, Position>(())
            .add_component_state::<HoverState, Size>(())
            .add_component_state::<HoverState, ColoredElement>(())
            .add_component_state::<HoverState, CornersRoundness>(())
            .add_component_state::<HoverState, FontSize>(())
            .add_systems(
                Update,
                (
                    transition_system::<Hovered<Position>>,
                    transition_system::<Hovered<Size>>,
                    transition_system::<Hovered<ColoredElement>>,
                    transition_system::<Hovered<CornersRoundness>>,
                    transition_system::<Hovered<FontSize>>,
                    remove_system::<Hovered<Position>>,
                    remove_system::<Hovered<Size>>,
                    remove_system::<Hovered<ColoredElement>>,
                    remove_system::<Hovered<CornersRoundness>>,
                    remove_system::<Hovered<FontSize>>,
                ),
            )
            .add_systems(
                Update,
                (
                    hover_effect_system::<Position>,
                    hover_effect_system::<Size>,
                    hover_effect_system::<ColoredElement>,
                    hover_effect_system::<CornersRoundness>,
                    hover_effect_system::<FontSize>,
                    hover_effect_clear_system::<Position>,
                    hover_effect_clear_system::<Size>,
                    hover_effect_clear_system::<ColoredElement>,
                    hover_effect_clear_system::<CornersRoundness>,
                    hover_effect_clear_system::<FontSize>,
                ),
            )
            .add_systems(
                Update,
                (
                    hover_effect_transition_in_system::<Position>,
                    hover_effect_transition_in_system::<Size>,
                    hover_effect_transition_in_system::<ColoredElement>,
                    hover_effect_transition_in_system::<CornersRoundness>,
                    hover_effect_transition_in_system::<FontSize>,
                    hover_effect_transition_out_system::<Position>,
                    hover_effect_transition_out_system::<Size>,
                    hover_effect_transition_out_system::<ColoredElement>,
                    hover_effect_transition_out_system::<CornersRoundness>,
                    hover_effect_transition_out_system::<FontSize>,
                ),
            );
    }
}

#[derive(Clone)]
pub struct HoverState;

pub type Hovered<T> = ComponentState<HoverState, T>;

#[derive(Component)]
pub struct HoverEffect<T: Component + Clone> {
    pub value: T,
}

impl<T: Component + Clone> HoverEffect<T> {
    pub fn new(value: T) -> HoverEffect<T> {
        HoverEffect { value }
    }
}

#[derive(Component)]
pub struct HoverEffectTransition<T: PropertyTransition<T> + Component + Clone> {
    value: T,

    in_duration: Duration,
    out_duration: Duration,
}

impl<T: PropertyTransition<T> + Component + Clone> HoverEffectTransition<T> {
    pub fn new(
        value: T,
        in_duration: Duration,
        out_duration: Duration,
    ) -> HoverEffectTransition<T> {
        HoverEffectTransition {
            value,

            in_duration,
            out_duration,
        }
    }
}

pub fn hover_effect_system<T: Component + Clone>(
    mut commands: Commands,
    mut events: EventReader<HoverEnterEvent>,
    effects: Query<&HoverEffect<T>>,
) {
    if !effects.is_empty() {
        for event in events.read() {
            let Ok(effect) = effects.get(event.element) else {
                continue;
            };

            commands
                .entity(event.element)
                .insert(Hovered::new(effect.value.clone()));
        }
    }
}

pub fn hover_effect_clear_system<T: Component + Clone>(
    mut commands: Commands,
    mut events: EventReader<HoverExitEvent>,
    effects: Query<(), (With<HoverEffect<T>>, With<Hovered<T>>)>,
) {
    if !effects.is_empty() {
        for event in events.read() {
            if !effects.contains(event.element) {
                continue;
            };

            commands.entity(event.element).remove::<Hovered<T>>();
        }
    }
}

pub fn hover_effect_transition_in_system<T: PropertyTransition<T> + Component + Clone>(
    world: &World,
    mut commands: Commands,
    mut events: EventReader<HoverEnterEvent>,
    effects: Query<(&HoverEffectTransition<T>, &T, Option<&Active<T>>)>,
) {
    if !effects.is_empty() {
        for event in events.read() {
            let Ok((effect, base_value, active_value)) = effects.get(event.element) else {
                continue;
            };

            let entity_ref = world.entity(event.element);

            commands
                .entity(event.element)
                .insert((
                    Hovered::new(
                        active_value
                            .active_or_base(world, &entity_ref, base_value)
                            .clone(),
                    ),
                    Transition::new(Hovered::new(effect.value.clone()), effect.in_duration),
                ))
                .remove::<AutoRemove<Hovered<T>>>();
        }
    }
}

pub fn hover_effect_transition_out_system<T: PropertyTransition<T> + Component + Clone>(
    world: &World,
    mut commands: Commands,
    mut events: EventReader<HoverExitEvent>,
    effects: Query<(&HoverEffectTransition<T>, &T, Option<&Active<T>>)>,
) {
    if !effects.is_empty() {
        for event in events.read() {
            let Ok((effect, base_value, active_value)) = effects.get(event.element) else {
                continue;
            };

            let entity_ref = world.entity(event.element);

            if active_value.is_active_state::<Hovered<T>>(world.components()) {
                commands.entity(event.element).insert((
                    Transition::new(
                        Hovered::new(
                            active_value
                                .second_active_or_base(world, &entity_ref, base_value)
                                .clone(),
                        ),
                        effect.out_duration,
                    ),
                    AutoRemove::<Hovered<T>>::new(effect.out_duration),
                ));
            } else {
                commands.entity(event.element).remove::<Hovered<T>>();
            }
        }
    }
}
