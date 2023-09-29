use bevy::{
    prelude::{Component, Entity, Parent, Plugin, Query, ReflectComponent, Update, With},
    reflect::Reflect,
    utils::HashMap,
};

use super::{Position, Size, VisibleRegion, ZLevel};

pub struct UiUpdatePropertiesPlugin;

impl Plugin for UiUpdatePropertiesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, (update_z, update_visible_region));
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

#[derive(Component, Debug, Default, Clone, Hash, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct AutoVisibleRegionUpdate;

fn update_visible_region(
    mut visible_regions: Query<
        (&mut VisibleRegion, Option<&Parent>),
        With<AutoVisibleRegionUpdate>,
    >,
    regions: Query<(&Position, &Size)>,
) {
    // TODO: Use events & filters to update VisibleRegion instead of updating always

    for (mut visible_region, parent) in visible_regions.iter_mut() {
        if let Some(parent) = parent {
            let Ok((parent_position, parent_size)) = regions.get(parent.get()) else {
                continue;
            };

            visible_region.x = parent_position.x;
            visible_region.y = parent_position.y;

            visible_region.width = parent_size.width;
            visible_region.height = parent_size.height;
        } else {
            visible_region.x = 0;
            visible_region.y = 0;

            visible_region.width = u32::MAX;
            visible_region.height = u32::MAX;
        }
    }
}
