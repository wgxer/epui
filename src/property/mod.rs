pub mod collision;
pub mod state;
pub mod transition;
pub mod update;

#[doc(hidden)]
pub mod prelude {
    pub use crate::property::{
        collision::{AABBCollisionBundle, BoxCollisionBundle},
        transition::Transition,
        update::AutoZUpdate,
        ColoredElement, CornersRoundness, Position, Size,
    };
}

use bevy::{
    prelude::{Color, Component, Rect, ReflectComponent, Vec2, Vec4},
    reflect::Reflect,
};

#[derive(Component, Debug, Default, Clone, Hash, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

impl Position {
    #[inline]
    pub fn new(x: u32, y: u32) -> Position {
        Position { x, y }
    }
}

impl From<Position> for Vec2 {
    fn from(position: Position) -> Self {
        Vec2::new(position.x as f32, position.y as f32)
    }
}

impl From<Vec2> for Position {
    fn from(position: Vec2) -> Self {
        Position::new(position.x as u32, position.y as u32)
    }
}

#[derive(Component, Debug, Default, Clone, Hash, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct ZLevel(pub u32);

#[derive(Component, Debug, Clone, Hash, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    #[inline]
    pub fn new(width: u32, height: u32) -> Size {
        Size { width, height }
    }
}

impl Default for Size {
    fn default() -> Self {
        Size::new(50, 50)
    }
}

impl From<Size> for Vec2 {
    fn from(size: Size) -> Self {
        Vec2::new(size.width as f32, size.height as f32)
    }
}

impl From<Vec2> for Size {
    fn from(size: Vec2) -> Self {
        Size::new(size.x as u32, size.y as u32)
    }
}

#[derive(Component, Debug, Default, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct ColoredElement {
    pub color: Color,
}

impl ColoredElement {
    #[inline]
    pub fn new(color: Color) -> ColoredElement {
        ColoredElement { color }
    }
}

#[derive(Component, Debug, Default, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct CornersRoundness {
    pub top_left_scalar: f32,
    pub top_right_scalar: f32,
    pub bottom_left_scalar: f32,
    pub bottom_right_scalar: f32,
}

impl CornersRoundness {
    pub fn from_scalar(scalar: f32) -> CornersRoundness {
        CornersRoundness {
            top_left_scalar: scalar,
            top_right_scalar: scalar,
            bottom_left_scalar: scalar,
            bottom_right_scalar: scalar,
        }
    }
}

impl From<CornersRoundness> for Vec4 {
    fn from(corners_roundness: CornersRoundness) -> Self {
        Vec4::new(
            corners_roundness.top_left_scalar,
            corners_roundness.top_right_scalar,
            corners_roundness.bottom_left_scalar,
            corners_roundness.bottom_right_scalar,
        )
    }
}

impl From<Vec4> for CornersRoundness {
    fn from(corners_roundness: Vec4) -> Self {
        CornersRoundness {
            top_left_scalar: corners_roundness.x,
            top_right_scalar: corners_roundness.y,
            bottom_left_scalar: corners_roundness.z,
            bottom_right_scalar: corners_roundness.w,
        }
    }
}

#[derive(Component, Debug, Default, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct VisibleRegion {
    pub x: u32,
    pub y: u32,

    pub width: u32,
    pub height: u32,
}

impl VisibleRegion {
    #[inline]
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> VisibleRegion {
        VisibleRegion {
            x,
            y,
            width,
            height,
        }
    }
}

impl From<Rect> for VisibleRegion {
    fn from(visible_region: Rect) -> Self {
        VisibleRegion {
            x: visible_region.min.x as u32,
            y: visible_region.min.y as u32,

            width: visible_region.width() as u32,
            height: visible_region.height() as u32,
        }
    }
}

impl From<VisibleRegion> for Rect {
    fn from(visible_region: VisibleRegion) -> Self {
        Rect::from_corners(
            Vec2::new(visible_region.x as f32, visible_region.y as f32),
            Vec2::new(
                visible_region.x as f32 + visible_region.width as f32,
                visible_region.y as f32 + visible_region.height as f32,
            ),
        )
    }
}
