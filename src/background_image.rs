use bevy::prelude::*;

use crate::{loading::TextureAssets, GameState};

pub struct BackgroundPlugin;

#[derive(Component)]
struct Background;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            OnTransition {
                from: GameState::Menu,
                to: GameState::Playing,
            },
            add_background,
        )
        .add_systems(
            OnTransition {
                from: GameState::Playing,
                to: GameState::Menu,
            },
            remove_background,
        );
    }
}

fn add_background(mut commands: Commands, textures: Res<TextureAssets>) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::splat(100000.0)),
                ..Default::default()
            },
            texture: textures.grass.clone(),
            transform: Transform::from_xyz(0.0, 0.0, -100.0),
            ..Default::default()
        },
        Background,
        ImageScaleMode::Tiled {
            tile_x: true,
            tile_y: true,
            stretch_value: 1.0,
        },
    ));
}

fn remove_background(mut commands: Commands, background: Query<Entity, With<Background>>) {
    for background in background.iter() {
        commands.entity(background).despawn();
    }
}
