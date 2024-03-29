use std::time::Duration;

use crate::{element::text::FontSize, property::*};
use bevy::{
    prelude::{
        Commands, Component, Entity, EventWriter, Plugin, PostUpdate, Query, Res, Vec2, Vec4,
    },
    time::{Time, Timer, TimerMode},
    window::RequestRedraw,
};

use super::state::ComponentState;

pub(crate) struct UiTransitionPlugin;

impl Plugin for UiTransitionPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            PostUpdate,
            (
                transition_system::<Position>,
                transition_system::<Size>,
                transition_system::<ColoredElement>,
                transition_system::<CornersRoundness>,
                transition_system::<FontSize>,
            ),
        );
    }
}

#[derive(Component)]
pub struct Transition<T: PropertyTransition<T> + Component + Clone> {
    from: Option<T>,
    to: T,

    timer: Timer,
}

impl<T: PropertyTransition<T> + Component + Clone> Transition<T> {
    pub fn new(to: T, duration: Duration) -> Transition<T> {
        Transition {
            from: None,
            to,
            timer: Timer::new(duration, TimerMode::Once),
        }
    }
}

pub fn transition_system<T: PropertyTransition<T> + Component + Clone>(
    mut commands: Commands,
    time: Res<Time>,
    mut transitions: Query<(Entity, &mut Transition<T>, &mut T)>,
    mut redraw_requester: EventWriter<RequestRedraw>,
) {
    let mut request_redraw = false;

    for (transition_entity, mut transition, mut transition_property) in transitions.iter_mut() {
        transition.timer.tick(time.delta());
        let progress = f32::min(1.0f32, transition.timer.fraction());

        let new_property_value = T::transition(
            progress,
            transition.from.as_ref().unwrap_or(&transition_property),
            &transition.to,
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
        Vec2::from(from.clone())
            .lerp(to.clone().into(), progress)
            .into()
    }
}

impl PropertyTransition<Size> for Size {
    fn transition<'a>(progress: f32, from: &'a Size, to: &'a Size) -> Size {
        Vec2::from(from.clone())
            .lerp(to.clone().into(), progress)
            .into()
    }
}

impl PropertyTransition<ColoredElement> for ColoredElement {
    fn transition<'a>(
        progress: f32,
        from: &'a ColoredElement,
        to: &'a ColoredElement,
    ) -> ColoredElement {
        ColoredElement::new(Color::rgba_from_array(
            from.color
                .rgba_to_vec4()
                .lerp(to.color.rgba_to_vec4(), progress),
        ))
    }
}

impl PropertyTransition<CornersRoundness> for CornersRoundness {
    fn transition<'a>(
        progress: f32,
        from: &'a CornersRoundness,
        to: &'a CornersRoundness,
    ) -> CornersRoundness {
        Vec4::from(from.clone())
            .lerp(to.clone().into(), progress)
            .into()
    }
}

impl PropertyTransition<FontSize> for FontSize {
    fn transition<'a>(progress: f32, from: &'a FontSize, to: &'a FontSize) -> FontSize {
        FontSize(
            Vec2::splat(from.0 as f32)
                .lerp(Vec2::splat(to.0 as f32), progress)
                .x
                .round() as u32,
        )
    }
}

impl<S: Send + Sync + 'static, T: Component + Clone + PropertyTransition<T>>
    PropertyTransition<ComponentState<S, T>> for ComponentState<S, T>
{
    fn transition<'a>(
        progress: f32,
        from: &'a ComponentState<S, T>,
        to: &'a ComponentState<S, T>,
    ) -> ComponentState<S, T> {
        ComponentState::new(T::transition(progress, from, to))
    }
}
