use std::time::Duration;

use bevy::{prelude::{Component, Vec2, Query, Vec4, Plugin, IntoSystemConfigs, CoreSet, Commands, Entity, EventWriter}, utils::Instant, window::RequestRedraw};
use crate::property::*;

pub struct UiTransitionPlugin;

impl Plugin for UiTransitionPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            (
                transition_system::<Position>, 
                transition_system::<Size>, 
                transition_system::<ColoredElement>, 
                transition_system::<CornersRoundness>, 
            ).in_base_set(CoreSet::PostUpdate)
        );
    }
}

#[derive(Component)]
pub struct Transition<T: PropertyTransition<T> + Component + Clone> {
    from: Option<T>,
    to: T,

    start_time: Instant, 
    duration: f32
}

impl <T: PropertyTransition<T> + Component + Clone> Transition<T> {
    pub fn new(to: T, duration: Duration) -> Transition<T> {
        Transition { from: None, to, start_time: Instant::now(), duration: duration.as_secs_f32() }
    }
}

pub(super) fn transition_system<T: PropertyTransition<T> + Component + Clone>(
    mut commands: Commands, 
    mut transitions: Query<(Entity, &mut Transition<T>, &mut T)>,
    mut redraw_requester: EventWriter<RequestRedraw>
) {
    let mut request_redraw = false;

    for (transition_entity, mut transition, mut transition_property) in transitions.iter_mut() {
        let elapsed = transition.start_time.elapsed().as_secs_f32();
        let progress = f32::min(1.0f32, elapsed / transition.duration);

        let new_property_value = T::transition(
            progress, 
            transition.from.as_ref().unwrap_or(&transition_property), 
            &transition.to
        );

        if transition.from.is_some() {
            *transition_property = new_property_value;
        } else {
            let from = std::mem::replace(transition_property.as_mut(), new_property_value);
            transition.from = Some(from);
        }

        if progress == 1.0f32 {
            if let Some(mut entity_commands) = commands.get_entity(transition_entity) {
                entity_commands.remove::<Transition<T>>();
            }
        } else {
            request_redraw = true;
        }
    }

    if request_redraw {
        redraw_requester.send(RequestRedraw);
    }
}

pub trait PropertyTransition<T: Component> {
    fn transition<'a>(progress: f32, from: &'a T, to: &'a T) -> T;
}

impl PropertyTransition<Position> for Position {
    fn transition<'a>(progress: f32, from: &'a Position, to: &'a Position) -> Position {
        Vec2::from(from.clone()).lerp(to.clone().into(), progress).into()
    }
}

impl PropertyTransition<Size> for Size {
    fn transition<'a>(progress: f32, from: &'a Size, to: &'a Size) -> Size {
        Vec2::from(from.clone()).lerp(to.clone().into(), progress).into()
    }
}

impl PropertyTransition<ColoredElement> for ColoredElement {
    fn transition<'a>(progress: f32, from: &'a ColoredElement, to: &'a ColoredElement) -> ColoredElement {
        ColoredElement::new(Vec4::from(from.color).lerp(to.color.into(), progress).into())
    }
}

impl PropertyTransition<CornersRoundness> for CornersRoundness {
    fn transition<'a>(progress: f32, from: &'a CornersRoundness, to: &'a CornersRoundness) -> CornersRoundness {
        Vec4::from(from.clone()).lerp(to.clone().into(), progress).into()
    }
}