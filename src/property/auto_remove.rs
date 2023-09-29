use std::{marker::PhantomData, time::Duration};

use crate::{element::text::FontSize, property::*};
use bevy::{
    prelude::{Commands, Component, Entity, EventWriter, Plugin, PostUpdate, Query, With, Without},
    utils::Instant,
    window::RequestRedraw,
};

pub(crate) struct UiAutoRemovePlugin;

impl Plugin for UiAutoRemovePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            PostUpdate,
            (
                remove_system::<Position>,
                remove_system::<Size>,
                remove_system::<ColoredElement>,
                remove_system::<CornersRoundness>,
                remove_system::<FontSize>,
            ),
        );
    }
}

#[derive(Component)]
pub struct AutoRemove<T: Component + Clone> {
    start_time: Instant,
    duration: f32,

    phantom: PhantomData<T>,
}

impl<T: Component + Clone> AutoRemove<T> {
    pub fn new(duration: Duration) -> AutoRemove<T> {
        AutoRemove {
            start_time: Instant::now(),
            duration: duration.as_secs_f32(),

            phantom: PhantomData,
        }
    }
}

pub fn remove_system<T: Component + Clone>(
    mut commands: Commands,
    mut redraw_requester: EventWriter<RequestRedraw>,
    removes: Query<(Entity, &AutoRemove<T>), With<T>>,
    invalid_removes: Query<Entity, (With<AutoRemove<T>>, Without<T>)>,
) {
    let mut request_redraw = false;

    for (remove_entity, remove) in removes.iter() {
        let elapsed = remove.start_time.elapsed().as_secs_f32();

        if elapsed > remove.duration {
            commands
                .entity(remove_entity)
                .remove::<(AutoRemove<T>, T)>();

            request_redraw = true;
        }
    }

    if request_redraw {
        redraw_requester.send(RequestRedraw);
    }

    for invalid_remove in invalid_removes.iter() {
        commands.entity(invalid_remove).remove::<AutoRemove<T>>();
    }
}
