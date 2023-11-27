use bevy::{
    prelude::{App, Color, Commands, Query, Startup, Update, With},
    DefaultPlugins,
};

use epui::prelude::*;

fn main() {
    App::new()
        .add_systems(Startup, setup)
        .add_systems(Update, (move_texts, change_font_size))
        .add_plugins(DefaultPlugins)
        .add_plugins(UiPlugin)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(UiCameraBundle::default());

    let mut bundles = Vec::new();

    for x in 0..20 {
        for y in 0..20 {
            bundles.push((UiTextBundle {
                text: UiText((x + 1).to_string()),

                color: ColoredElement::new(Color::RED),

                position: Position::new(x * 40, y * 40),
                size: Size::new(36, 36),

                ..Default::default()
            },));
        }
    }

    commands.spawn_batch(bundles);
}

fn move_texts(mut texts: Query<&mut Position, With<UiText>>) {
    for mut pos in texts.iter_mut() {
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

fn change_font_size(mut texts: Query<&mut FontSize, With<UiText>>) {
    for mut font_size in texts.iter_mut() {
        if font_size.0 >= 16 {
            font_size.0 = 8;
        } else {
            font_size.0 += 1;
        }
    }
}
