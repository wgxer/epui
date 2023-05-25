use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use bevy::{
    ecs::{
        component::{ComponentId, Components},
        world::{EntityMut, EntityRef},
    },
    prelude::{
        Added, App, Commands, Component, CoreSet, Entity, IntoSystemConfig, Mut, Query,
        RemovedComponents, Res, Resource,
    },
};

#[derive(Component, Clone)]
pub struct ComponentState<S: Send + Sync + 'static, T: Component + Clone> {
    pub value: T,
    phantom: PhantomData<S>,
}

impl<S: Send + Sync + 'static, T: Component + Clone> ComponentState<S, T> {
    pub fn new(value: T) -> Self {
        ComponentState {
            value,
            phantom: PhantomData,
        }
    }
}

impl<S: Send + Sync + 'static, T: Component + Clone> Deref for ComponentState<S, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<S: Send + Sync + 'static, T: Component + Clone> DerefMut for ComponentState<S, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

#[derive(Component, Clone)]
pub struct Active<T: Component + Clone> {
    component_ids: Vec<ComponentId>,
    phantom: PhantomData<T>,
}

impl<T: Component + Clone> Active<T> {
    pub fn new(components: &Components) -> Self {
        let component_id = components
            .component_id::<T>()
            .expect("Couldn't get component id of component");

        Active {
            component_ids: vec![component_id],
            phantom: PhantomData,
        }
    }

    pub fn is_active_state<C: Component>(&self, components: &Components) -> bool {
        components.component_id::<C>() == self.component_ids.last().copied()
    }

    pub fn get_active<'a>(&self, entity_ref: &'a EntityRef) -> Option<&'a T> {
        if let Some(active) = self.component_ids.last().copied() {
            Some(Self::get_by_component_id(entity_ref, active))
        } else {
            None
        }
    }

    pub fn active<'a>(&self, entity_ref: &'a EntityRef) -> &'a T {
        if let Some(active) = self.component_ids.last().copied() {
            Self::get_by_component_id(entity_ref, active)
        } else {
            panic!("Couldn't find any active component, did this state get created correctly?");
        }
    }

    pub fn active_mut<'a>(&mut self, entity_mut: &'a mut EntityMut) -> Mut<'a, T> {
        if let Some(active) = self.component_ids.last().copied() {
            Self::get_mut_by_component_id(entity_mut, active)
        } else {
            panic!("Couldn't find any active component, did this state get created correctly?");
        }
    }

    pub fn get_state<'a>(&self, entity_ref: &'a EntityRef, index: usize) -> Option<&'a T> {
        self.component_ids
            .get(index)
            .copied()
            .map(|component_id| Self::get_by_component_id(entity_ref, component_id))
    }

    pub fn get_state_mut<'a>(
        &self,
        entity_mut: &'a mut EntityMut,
        index: usize,
    ) -> Option<Mut<'a, T>> {
        self.component_ids
            .get(index)
            .copied()
            .map(|component_id| Self::get_mut_by_component_id(entity_mut, component_id))
    }

    pub fn states_len(&self) -> usize {
        self.component_ids.len()
    }

    fn get_by_component_id<'a>(entity_ref: &'a EntityRef, component_id: ComponentId) -> &'a T {
        if Some(component_id) == entity_ref.world().component_id::<T>() {
            entity_ref.get::<T>().expect(
                "Couldn't find requested component, did correct entity reference get passed?",
            )
        } else {
            unsafe {
                entity_ref
                    .get_by_id(component_id)
                    .expect(
                        "Couldn't find requested component, did correct entity reference get passed?",
                    )
                    .deref::<ComponentState<(), T>>()
            }
        }
    }

    fn get_mut_by_component_id<'a>(
        entity_mut: &'a mut EntityMut,
        component_id: ComponentId,
    ) -> Mut<'a, T> {
        if Some(component_id) == entity_mut.world().component_id::<T>() {
            entity_mut.get_mut::<T>().expect(
                "Couldn't find requested component, did correct entity reference get passed?",
            )
        } else {
            unsafe {
                entity_mut
                    .get_mut_by_id(component_id)
                    .unwrap()
                    .with_type::<ComponentState<(), T>>()
            }
            .map_unchanged(|x| x.deref_mut())
        }
    }
}

pub trait ActiveOptionExt<T: Component + Clone> {
    fn is_active_state<C: Component>(&self, components: &Components) -> bool;
    fn active_or_base<'a>(&self, entity_ref: &'a EntityRef, base: &'a T) -> &'a T;
}

impl<T: Component + Clone> ActiveOptionExt<T> for Option<&Active<T>> {
    fn is_active_state<C: Component>(&self, components: &Components) -> bool {
        self.map(|active| active.is_active_state::<C>(components))
            .unwrap_or(false)
    }

    fn active_or_base<'a>(&self, entity_ref: &'a EntityRef, base: &'a T) -> &'a T {
        self.map(|active| active.active(entity_ref)).unwrap_or(base)
    }
}

#[derive(Resource)]
pub struct ComponentStates<T: Component + Clone> {
    component_ids: Vec<ComponentId>,
    phantom: PhantomData<T>,
}

fn add_component_state<S: Send + Sync + 'static, T: Component + Clone>(
    mut commands: Commands,
    components: &Components,
    states: Res<ComponentStates<T>>,
    mut active_components: Query<(Entity, Option<&mut Active<T>>), Added<ComponentState<S, T>>>,
) {
    for (entity, active) in &mut active_components {
        if let Some(mut active) = active {
            active.component_ids.push(
                components.component_id::<ComponentState<S, T>>().expect(
                    format!(
                        "Failed to get component id for type: {:?}",
                        std::any::type_name::<ComponentState<S, T>>(),
                    )
                    .as_str(),
                ),
            );

            active.component_ids.sort_unstable_by_key(|component_id| {
                usize::MAX
                    - states
                        .component_ids
                        .iter()
                        .position(|x| x == component_id)
                        .unwrap_or_default()
            });
        } else {
            let mut active = Active::<T>::new(components);

            bevy::log::info!("new active<T>");

            active.component_ids.push(
                components.component_id::<ComponentState<S, T>>().expect(
                    format!(
                        "Failed to get component id for type: {:?}",
                        std::any::type_name::<ComponentState<S, T>>(),
                    )
                    .as_str(),
                ),
            );

            commands.entity(entity).insert(active);
        }
    }
}

fn remove_component_state<S: Send + Sync + 'static, T: Component + Clone>(
    components: &Components,
    mut active_components: Query<&mut Active<T>>,
    mut removed_states: RemovedComponents<ComponentState<S, T>>,
) {
    let mut active_components = active_components.iter_many_mut(removed_states.into_iter());

    while let Some(mut active) = active_components.fetch_next() {
        active.component_ids.retain(|component_id| {
            *component_id
                != components.component_id::<ComponentState<S, T>>().expect(
                    format!(
                        "Failed to get component id for type: {:?}",
                        std::any::type_name::<ComponentState<S, T>>(),
                    )
                    .as_str(),
                )
        });
    }
}

pub trait AppComponentStateExt {
    fn add_component_state<S: Send + Sync + 'static, T: Component + Clone>(
        &mut self,
        _: (),
    ) -> &mut Self;
}

impl AppComponentStateExt for App {
    fn add_component_state<S: Send + Sync + 'static, T: Component + Clone>(
        &mut self,
        _: (),
    ) -> &mut Self {
        if !self.world.contains_resource::<ComponentStates<T>>() {
            let component_id = match self.world.component_id::<T>() {
                Some(component_id) => component_id,
                None => self.world.init_component::<T>(),
            };

            self.insert_resource(ComponentStates::<T> {
                component_ids: vec![component_id],
                phantom: PhantomData,
            });
        }

        if self.world.component_id::<ComponentState<S, T>>().is_none() {
            self.world.init_component::<ComponentState<S, T>>();
        }

        self.add_systems((
            add_component_state::<S, T>,
            remove_component_state::<S, T>.in_base_set(CoreSet::PostUpdate),
        ));

        self
    }
}
