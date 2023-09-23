use bevy::{
    prelude::{App, Color, Commands, IntoSystemAppConfig},
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

    commands.spawn(UiTextBundle {
        text: UiText(String::from("Hello world !")),
        font_size: FontSize(32),
        color: ColoredElement::new(Color::RED),

        position: Position::new(50, 50),
        size: Size::new(500, 50),

        ..Default::default()
    });
}
