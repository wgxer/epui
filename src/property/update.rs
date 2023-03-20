use bevy::{
    prelude::{Component, Entity, Parent, Plugin, Query, ReflectComponent},
    reflect::Reflect,
    utils::HashMap,
};

use super::ZLevel;

pub struct UiUpdatePropertiesPlugin;

impl Plugin for UiUpdatePropertiesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(update_z);
    }
}

#[derive(Component, Debug, Default, Clone, Hash, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct AutoZUpdate;

fn update_z(mut z_levels: Query<(Entity, &mut ZLevel, Option<(&AutoZUpdate, &Parent)>)>) {
    // TODO: Use events to update ZLevel instead of updating always

    let mut z_levels_map = HashMap::new();

    for (entity, z_level, _) in z_levels.iter() {
        z_levels_map.insert(entity, z_level.0);
    }

    for (_, mut z_level, auto_update_data) in z_levels.iter_mut() {
        if let Some((_, parent)) = auto_update_data {
            if let Some(parent_z_level) = z_levels_map.get(&parent.get()) {
                z_level.0 = *parent_z_level + 1;
            }
        }
    }
}
