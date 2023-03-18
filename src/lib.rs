pub mod r#box;
pub mod camera;
pub mod property;
pub mod transition;

#[doc(hidden)]
pub mod prelude {
    pub use crate::UiPlugin;
    pub use crate::camera::{UiCamera, UiCameraBundle};

    pub use crate::property::*;
    pub use crate::r#box::{UiBox, UiBoxBundle};
}

use bevy::{prelude::*, window::RequestRedraw, winit::WinitSettings};

use r#box::UiBoxPlugin;
use transition::UiTransitionPlugin;

use crate::camera::UiCameraPlugin;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(redraw.on_startup())
            .add_plugin(UiCameraPlugin)
            .add_plugin(UiBoxPlugin)
            .add_plugin(UiTransitionPlugin)
            .insert_resource(WinitSettings::desktop_app());
    }
}

fn redraw(mut request_redraw_writer: EventWriter<RequestRedraw>) {
    request_redraw_writer.send(RequestRedraw);
}
