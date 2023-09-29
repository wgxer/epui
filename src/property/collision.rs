use std::any::Any;

use bevy::{
    math::Vec2Swizzles,
    prelude::{Bundle, Changed, Component, IVec2, Plugin, Query, Rect, Update, Vec2},
    reflect::Reflect,
};

use super::{CornersRoundness, Position, Size};

pub struct UiCollisionPlugin;

impl Plugin for UiCollisionPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, update_box_collision);
    }
}

#[derive(Component)]
pub struct Collision(pub Box<dyn CollisionFunction + Send + Sync>);

#[derive(Component, Debug, Default, Clone, Hash, PartialEq, Eq, Reflect)]
pub struct BoxCollision;

#[derive(Bundle)]
pub struct BoxCollisionBundle {
    collision: Collision,
    box_collision: BoxCollision,
}

impl BoxCollisionBundle {
    pub fn new() -> BoxCollisionBundle {
        BoxCollisionBundle::default()
    }
}

impl Default for BoxCollisionBundle {
    fn default() -> Self {
        BoxCollisionBundle {
            collision: Collision(Box::new(BoxCollisionFunction::default())),
            box_collision: BoxCollision,
        }
    }
}

#[derive(Bundle)]
pub struct AABBCollisionBundle {
    collision: Collision,
}

impl AABBCollisionBundle {
    pub fn new() -> AABBCollisionBundle {
        AABBCollisionBundle::default()
    }
}

impl Default for AABBCollisionBundle {
    fn default() -> Self {
        AABBCollisionBundle {
            collision: Collision(Box::new(AABBCollisionFunction)),
        }
    }
}

fn update_box_collision(
    mut query: Query<(&mut Collision, &CornersRoundness), Changed<CornersRoundness>>,
) {
    for (mut collision, corners_roundness) in query.iter_mut() {
        if let Some(box_collision) = collision
            .0
            .as_any_mut()
            .downcast_mut::<BoxCollisionFunction>()
        {
            box_collision.corners_roundness = corners_roundness.clone();
        } else {
            collision.0 = Box::new(BoxCollisionFunction {
                corners_roundness: corners_roundness.clone(),
            });
        }
    }
}

pub trait CollisionFunction {
    fn contains(&self, position: Position, size: Size, point: Vec2) -> bool;

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct AABBCollisionFunction;

impl CollisionFunction for AABBCollisionFunction {
    fn contains(&self, position: Position, size: Size, point: Vec2) -> bool {
        Rect::from_corners(
            position.clone().into(),
            Vec2::from(position.clone()) + Vec2::from(size.clone()),
        )
        .contains(point)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Default)]
pub struct BoxCollisionFunction {
    pub corners_roundness: CornersRoundness,
}

impl CollisionFunction for BoxCollisionFunction {
    fn contains(&self, position: Position, size: Size, point: Vec2) -> bool {
        let aabb_rect = Rect::from_corners(
            position.clone().into(),
            Vec2::from(position.clone()) + Vec2::from(size.clone()),
        );

        if !aabb_rect.contains(point) {
            return false;
        }

        if self.corners_roundness == CornersRoundness::from_scalar(0.0f32) {
            return true;
        }

        let max_axis = if size.width >= size.height {
            Vec2::X
        } else {
            Vec2::Y
        };

        let min_axis = max_axis.yx();
        let half_axis_difference = size.width.abs_diff(size.height) as f32 / 2.0f32;

        let center = aabb_rect.center();

        let center_rect = Rect::from_center_half_size(
            center,
            ((min_axis * Vec2::from(size)) / 2.0f32) + (max_axis * half_axis_difference),
        );

        if center_rect.contains(point) {
            return true;
        }

        let min_axis_value = aabb_rect.half_size().min_element();
        let check_corner_axes = (point - center) / (point - center).abs();

        let corner_roundness = match check_corner_axes.as_ivec2() {
            IVec2 {
                x: -1 | 0,
                y: -1 | 0,
            } => self.corners_roundness.top_left_scalar,
            IVec2 { x: 1, y: -1 | 0 } => self.corners_roundness.top_right_scalar,
            IVec2 { x: -1 | 0, y: 1 } => self.corners_roundness.bottom_left_scalar,
            IVec2 { x: 1, y: 1 } => self.corners_roundness.bottom_right_scalar,
            _ => {
                bevy::prelude::warn!("Unknown check corner axes: {:?}", check_corner_axes);
                0.0f32
            }
        };

        let corner_shift = check_corner_axes * min_axis_value * (1.0f32 - corner_roundness);
        let check_roundness_distance = min_axis_value * corner_roundness;

        let point_shift_abs = (point - center).abs();
        let check_roundness_point_shift = (max_axis * half_axis_difference + corner_shift).abs();

        if point_shift_abs.x < check_roundness_point_shift.x
            || point_shift_abs.y < check_roundness_point_shift.y
        {
            return true;
        }

        if point_shift_abs.distance(check_roundness_point_shift) <= check_roundness_distance {
            return true;
        }

        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
