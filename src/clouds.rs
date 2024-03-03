use bevy::prelude::*;

use crate::{loading::TextureAssets, GameState};

pub struct CloudPlugin;

#[derive(Component)]
pub struct Cloud;

impl Plugin for CloudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnTransition {
                from: GameState::Menu,
                to: GameState::Playing,
            },
            spawn_clouds,
        )
        .add_systems(Update, move_clouds.run_if(in_state(GameState::Playing)));
    }
}

fn spawn_clouds(mut commands: Commands, textures: Res<TextureAssets>) {
    commands.spawn((
        SpriteBundle {
            texture: textures.cloud_1.clone(),
            transform: Transform::from_xyz(-150.0, -500.0, -10.0),
            ..Default::default()
        },
        Cloud,
    ));
}

fn move_clouds(mut clouds: Query<&mut Transform, With<Cloud>>, time: Res<Time>) {
    let dt = time.delta_seconds();
    for mut cloud in clouds.iter_mut() {
        cloud.translation -= Vec3::new(-10.0, -10.0, 0.0) * dt;
    }
}
