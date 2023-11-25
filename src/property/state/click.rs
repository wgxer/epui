use std::time::Duration;

use bevy::prelude::{Commands, Component, EventReader, Plugin, Query, Update, With, World};

use crate::{
    event::{PressEvent, ReleaseEvent},
    prelude::*,
    property::{
        auto_remove::{remove_system, AutoRemove},
        transition::{transition_system, PropertyTransition},
    },
};

use super::{Active, ActiveOptionExt, AppComponentStateExt, ComponentState};

pub struct UiClickStatePlugin;

impl Plugin for UiClickStatePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_component_state::<ClickState, Position>(())
            .add_component_state::<ClickState, Size>(())
            .add_component_state::<ClickState, ColoredElement>(())
            .add_component_state::<ClickState, CornersRoundness>(())
            .add_component_state::<ClickState, FontSize>(())
            .add_systems(
                Update,
                (
                    transition_system::<Clicked<Position>>,
                    transition_system::<Clicked<Size>>,
                    transition_system::<Clicked<ColoredElement>>,
                    transition_system::<Clicked<CornersRoundness>>,
                    transition_system::<Clicked<FontSize>>,
                    remove_system::<Clicked<Position>>,
                    remove_system::<Clicked<Size>>,
                    remove_system::<Clicked<ColoredElement>>,
                    remove_system::<Clicked<CornersRoundness>>,
                    remove_system::<Clicked<FontSize>>,
                ),
            )
            .add_systems(
                Update,
                (
                    click_effect_system::<Position>,
                    click_effect_system::<Size>,
                    click_effect_system::<ColoredElement>,
                    click_effect_system::<CornersRoundness>,
                    click_effect_system::<FontSize>,
                    click_effect_clear_system::<Position>,
                    click_effect_clear_system::<Size>,
                    click_effect_clear_system::<ColoredElement>,
                    click_effect_clear_system::<CornersRoundness>,
                    click_effect_clear_system::<FontSize>,
                ),
            )
            .add_systems(
                Update,
                (
                    click_effect_transition_in_system::<Position>,
                    click_effect_transition_in_system::<Size>,
                    click_effect_transition_in_system::<ColoredElement>,
                    click_effect_transition_in_system::<CornersRoundness>,
                    click_effect_transition_in_system::<FontSize>,
                    click_effect_transition_out_system::<Position>,
                    click_effect_transition_out_system::<Size>,
                    click_effect_transition_out_system::<ColoredElement>,
                    click_effect_transition_out_system::<CornersRoundness>,
                    click_effect_transition_out_system::<FontSize>,
                ),
            );
    }
}

#[derive(Clone)]
pub struct ClickState;

pub type Clicked<T> = ComponentState<ClickState, T>;

#[derive(Component)]
pub struct ClickEffect<T: Component + Clone> {
    pub value: T,
}

impl<T: Component + Clone> ClickEffect<T> {
    pub fn new(value: T) -> ClickEffect<T> {
        ClickEffect { value }
    }
}

#[derive(Component)]
pub struct ClickEffectTransition<T: PropertyTransition<T> + Component + Clone> {
    value: T,

    in_duration: Duration,
    out_duration: Duration,
}

impl<T: PropertyTransition<T> + Component + Clone> ClickEffectTransition<T> {
    pub fn new(
        value: T,
        in_duration: Duration,
        out_duration: Duration,
    ) -> ClickEffectTransition<T> {
        ClickEffectTransition {
            value,

            in_duration,
            out_duration,
        }
    }
}

pub fn click_effect_system<T: Component + Clone>(
    mut commands: Commands,
    mut events: EventReader<PressEvent>,
    effects: Query<&ClickEffect<T>>,
) {
    if !effects.is_empty() {
        for event in events.read() {
            let Ok(effect) = effects.get(event.element) else {
                continue;
            };

            commands
                .entity(event.element)
                .insert(Clicked::new(effect.value.clone()));
        }
    }
}

pub fn click_effect_clear_system<T: Component + Clone>(
    mut commands: Commands,
    mut events: EventReader<ReleaseEvent>,
    effects: Query<(), (With<ClickEffect<T>>, With<Clicked<T>>)>,
) {
    if !effects.is_empty() {
        for event in events.read() {
            if !effects.contains(event.element) {
                continue;
            };

            commands.entity(event.element).remove::<Clicked<T>>();
        }
    }
}

pub fn click_effect_transition_in_system<T: PropertyTransition<T> + Component + Clone>(
    world: &World,
    mut commands: Commands,
    mut events: EventReader<PressEvent>,
    effects: Query<(&ClickEffectTransition<T>, &T, Option<&Active<T>>)>,
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
                    Clicked::new(
                        active_value
                            .active_or_base(world, &entity_ref, base_value)
                            .clone(),
                    ),
                    Transition::new(Clicked::new(effect.value.clone()), effect.in_duration),
                ))
                .remove::<AutoRemove<Clicked<T>>>();
        }
    }
}

pub fn click_effect_transition_out_system<T: PropertyTransition<T> + Component + Clone>(
    world: &World,
    mut commands: Commands,
    mut events: EventReader<ReleaseEvent>,
    effects: Query<(&ClickEffectTransition<T>, &T, Option<&Active<T>>)>,
) {
    if !effects.is_empty() {
        for event in events.read() {
            let Ok((effect, base_value, active_value)) = effects.get(event.element) else {
                continue;
            };

            let entity_ref = world.entity(event.element);

            if active_value.is_active_state::<Clicked<T>>(world.components()) {
                commands.entity(event.element).insert((
                    Transition::new(
                        Clicked::new(
                            active_value
                                .second_active_or_base(world, &entity_ref, base_value)
                                .clone(),
                        ),
                        effect.out_duration,
                    ),
                    AutoRemove::<Clicked<T>>::new(effect.out_duration),
                ));
            } else {
                commands.entity(event.element).remove::<Clicked<T>>();
            }
        }
    }
}
