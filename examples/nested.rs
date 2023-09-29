use bevy::{
    prelude::{App, BuildChildren, Color, Commands, Startup},
    DefaultPlugins,
};

use epui::prelude::*;

fn main() {
    App::new()
        .add_systems(Startup, setup)
        .add_plugins(DefaultPlugins)
        .add_plugins(UiPlugin)
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
                            position: Position::new(235, 235),
                            size: Size::new(50, 50),
                            color: ColoredElement::new(Color::BLUE),
                            ..Default::default()
                        },
                        CornersRoundness::from_scalar(1.0f32),
                    ));

                    for (x, y) in [(85, 235), (385, 235), (235, 85), (235, 385)] {
                        parent.spawn((
                            UiBoxBundle {
                                position: Position::new(x, y),
                                size: Size::new(50, 50),
                                color: ColoredElement::new(Color::BLACK),
                                ..Default::default()
                            },
                            CornersRoundness::from_scalar(0.25f32),
                        ));
                    }
                });
        });
}
