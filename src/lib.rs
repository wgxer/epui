pub mod camera;
pub mod element;
pub mod event;
pub mod property;

#[doc(hidden)]
pub mod prelude {
    pub use crate::camera::{UiCamera, UiCameraBundle};
    pub use crate::UiPlugin;

    pub use crate::element::prelude::*;
    pub use crate::property::prelude::*;
}

use bevy::{prelude::*, window::RequestRedraw, winit::WinitSettings};

use element::{r#box::UiBoxPlugin, text::UiTextPlugin};
use event::UiEventPlugin;
use property::{
    auto_remove::UiAutoRemovePlugin,
    collision::UiCollisionPlugin,
    state::{click::UiClickStatePlugin, hover::UiHoverStatePlugin},
    transition::UiTransitionPlugin,
    update::UiUpdatePropertiesPlugin,
};

use crate::camera::UiCameraPlugin;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, redraw)
            .add_plugins(UiCameraPlugin)
            .add_plugins(UiBoxPlugin)
            .add_plugins(UiTextPlugin)
            .add_plugins(UiTransitionPlugin)
            .add_plugins(UiAutoRemovePlugin)
            .add_plugins(UiCollisionPlugin)
            .add_plugins(UiUpdatePropertiesPlugin)
            .add_plugins(UiEventPlugin)
            .add_plugins(UiHoverStatePlugin)
            .add_plugins(UiClickStatePlugin)
            .insert_resource(WinitSettings::game());
    }
}

fn redraw(mut request_redraw_writer: EventWriter<RequestRedraw>) {
    request_redraw_writer.send(RequestRedraw);
}
