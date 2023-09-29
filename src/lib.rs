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
        app.add_system(redraw.on_startup())
            .add_plugin(UiCameraPlugin)
            .add_plugin(UiBoxPlugin)
            .add_plugin(UiTextPlugin)
            .add_plugin(UiTransitionPlugin)
            .add_plugin(UiAutoRemovePlugin)
            .add_plugin(UiCollisionPlugin)
            .add_plugin(UiUpdatePropertiesPlugin)
            .add_plugin(UiEventPlugin)
            .add_plugin(UiHoverStatePlugin)
            .add_plugin(UiClickStatePlugin)
            .insert_resource(WinitSettings::desktop_app());
    }
}

fn redraw(mut request_redraw_writer: EventWriter<RequestRedraw>) {
    request_redraw_writer.send(RequestRedraw);
}
