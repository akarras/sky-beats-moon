use bevy::prelude::*;

use crate::{loading::TextureAssets, player::Player, GameState};

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
        )
        .add_systems(
            Update,
            shift_background.run_if(in_state(GameState::Playing)),
        );
    }
}

const BACKGROUND_SIZE: f32 = 512.0;
const TILE_STRETCH: f32 = 2.0;

/// Keeps the background in view of the camera, but shifts it so that the tiled texture always lines up.
fn shift_background(
    mut background: Query<&mut Transform, With<Background>>,
    player: Query<&Transform, (Without<Background>, With<Player>)>,
) {
    for player in player.iter() {
        for mut background in background.iter_mut() {
            let player_location = player.translation;
            let x = player_location.x / (BACKGROUND_SIZE * TILE_STRETCH);
            let y = player_location.y / (BACKGROUND_SIZE * TILE_STRETCH);
            let nearest_x = x.round();
            let nearest_y = y.round();
            background.translation.x = nearest_x * BACKGROUND_SIZE * TILE_STRETCH;
            background.translation.y = nearest_y * BACKGROUND_SIZE * TILE_STRETCH;
        }
    }
}

fn add_background(mut commands: Commands, textures: Res<TextureAssets>) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::splat(10000.0)),
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
            stretch_value: TILE_STRETCH,
        },
    ));
}

fn remove_background(mut commands: Commands, background: Query<Entity, With<Background>>) {
    for background in background.iter() {
        commands.entity(background).despawn();
    }
}
