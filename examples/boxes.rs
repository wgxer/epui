use bevy::{
    prelude::{App, Color, Commands, Startup},
    DefaultPlugins,
};

use epui::{prelude::*, property::state::hover::HoverEffect};

fn main() {
    App::new()
        .add_systems(Startup, setup)
        .add_plugins(DefaultPlugins)
        .add_plugins(UiPlugin)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(UiCameraBundle::default());

    let mut bundles = Vec::new();

    for x in 0..50 {
        for y in 0..50 {
            bundles.push((
                UiBoxBundle {
                    color: ColoredElement::new(Color::RED),

                    position: Position::new(x * 20, y * 20),
                    size: Size::new(18, 18),

                    ..Default::default()
                },
                HoverEffect::new(ColoredElement::new(Color::DARK_GREEN)),
                AABBCollisionBundle::default(),
            ));
        }
    }

    commands.spawn_batch(bundles);
}
