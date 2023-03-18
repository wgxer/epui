use std::time::Duration;

use bevy::{
    prelude::{App, Color, Commands, IntoSystemAppConfig},
    DefaultPlugins,
};

use epui::{
    camera::UiCameraBundle,
    property::{ColoredElement, CornersRoundness, Position, Size},
    r#box::UiBoxBundle,
    UiPlugin, transition::Transition,
};

fn main() {
    App::new()
        .add_system(setup.on_startup())
        .add_plugins(DefaultPlugins)
        .add_plugin(UiPlugin)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(UiCameraBundle::default());

    for x in 1..=10 {
        for y in 1..=10 {
            commands.spawn((
                UiBoxBundle {
                    position: Position::new(x * 60, y * 60),
                    size: Size::new(50, 50),
                    color: ColoredElement::new(Color::hsl(
                        (x as f32 / 10.0) * 360.0,
                        y as f32 / 10.0,
                        0.5,
                    )),
                    ..Default::default()
                },
                CornersRoundness::from_scalar(0.0f32),
                Transition::new(
                    CornersRoundness::from_scalar(0.5f32), 
                    Duration::from_millis(1000)
                )
            ));
        }
    }
}
