use bevy::{
    prelude::{App, Color, Commands, Query, Startup, Update, With},
    DefaultPlugins,
};

use epui::prelude::*;

fn main() {
    App::new()
        .add_systems(Startup, setup)
        .add_systems(Update, move_boxes)
        .add_plugins(DefaultPlugins)
        .add_plugins(UiPlugin)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(UiCameraBundle::default());

    let mut bundles = Vec::new();

    for x in 0..50 {
        for y in 0..50 {
            bundles.push((UiBoxBundle {
                color: ColoredElement::new(Color::RED),

                position: Position::new(x * 20, y * 20),
                size: Size::new(18, 18),

                ..Default::default()
            },));
        }
    }

    commands.spawn_batch(bundles);
}

fn move_boxes(mut boxes: Query<&mut Position, With<UiBox>>) {
    for mut pos in boxes.iter_mut() {
        if pos.x % 10 == 9 {
            pos.x -= 9;
        } else {
            pos.x += 1;
        }

        if pos.y % 10 == 9 {
            pos.y -= 9;
        } else {
            pos.y += 1;
        }
    }
}
