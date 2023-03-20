use bevy::{
    prelude::{App, BuildChildren, Color, Commands, IntoSystemAppConfig},
    DefaultPlugins,
};

use epui::prelude::*;

fn main() {
    App::new()
        .add_system(setup.on_startup())
        .add_plugins(DefaultPlugins)
        .add_plugin(UiPlugin)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(UiCameraBundle::default());

    commands
        .spawn(UiBoxBundle {
            position: Position::new(10, 10),
            size: Size::new(500, 500),
            color: ColoredElement::new(Color::RED),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    UiBoxBundle {
                        position: Position::new(110, 110),
                        size: Size::new(300, 300),
                        color: ColoredElement::new(Color::GREEN),
                        ..Default::default()
                    },
                    CornersRoundness::from_scalar(0.5f32),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        UiBoxBundle {
                            position: Position::new(200, 200),
                            size: Size::new(50, 50),
                            color: ColoredElement::new(Color::BLUE),
                            ..Default::default()
                        },
                        CornersRoundness::from_scalar(1.0f32),
                    ));
                });
        });
}
