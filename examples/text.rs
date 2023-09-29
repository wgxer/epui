use bevy::{
    prelude::{App, Color, Commands, Startup},
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

    commands.spawn(UiTextBundle {
        text: UiText(String::from("Hello world !")),
        font_size: FontSize(32),
        color: ColoredElement::new(Color::RED),

        position: Position::new(50, 50),
        size: Size::new(500, 50),

        ..Default::default()
    });
}
