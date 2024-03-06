use std::ops::Neg;

use bevy::prelude::*;

use crate::{actions::Actions, player::Player, weapon::Velocity, GameState};

pub struct FollowCameraPlugin;

/// Follows the [`Player`] around
#[derive(Component)]
pub struct FollowCam;

impl Plugin for FollowCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (scale_camera, follow_camera).run_if(in_state(GameState::Playing)),
        );
    }
}

fn scale_camera(
    mut cameras: Query<&mut OrthographicProjection, With<Camera>>,
    actions: Res<Actions>,
) {
    if let Some(scale) = actions.camera_zoom {
        for mut camera in &mut cameras {
            camera.scale += scale.clamp(-0.2, 0.2);
            camera.scale = camera.scale.clamp(0.5, 2.5);
        }
    }
}

fn follow_camera(
    mut camera: Query<&mut Transform, With<Camera>>,
    players: Query<(&Transform, &Velocity), (With<Player>, Without<Camera>)>,
) {
    for (player, velocity) in &players {
        for mut camera in &mut camera {
            let offset = velocity.0.normalize().neg() * 10.0;
            camera.translation = player.translation + Vec3::new(offset.x, offset.y, 0.0);
        }
    }
}
