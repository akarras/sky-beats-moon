use bevy::prelude::*;
use bevy_rand::{prelude::WyRand, resource::GlobalEntropy};
use rand::Rng;

use crate::{loading::TextureAssets, weapon::Velocity, GameState};

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

fn spawn_clouds(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    mut rand: ResMut<GlobalEntropy<WyRand>>,
) {
    let spawn_range = 20000.0;
    let wind_speed = 200.0;
    let number_of_clouds = rand.gen_range(100..1000);
    let clouds = (0..number_of_clouds)
        .into_iter()
        .map(|_i| {
            (
                SpriteBundle {
                    texture: textures.cloud_1.clone(),
                    transform: Transform::from_xyz(
                        rand.gen_range(-spawn_range..spawn_range),
                        rand.gen_range(-spawn_range..spawn_range),
                        -10.0,
                    )
                    .with_scale(Vec3::new(0.8, 0.8, 0.8)),
                    ..Default::default()
                },
                Cloud,
                Velocity(Vec2::new(
                    rand.gen_range(-wind_speed..wind_speed),
                    rand.gen_range(-wind_speed..wind_speed),
                )),
            )
        })
        .collect::<Vec<_>>();
    commands.spawn_batch(clouds);
}

fn move_clouds(mut clouds: Query<&mut Transform, With<Cloud>>, time: Res<Time>) {
    let dt = time.delta_seconds();
    for mut cloud in clouds.iter_mut() {
        cloud.translation -= Vec3::new(-10.0, -10.0, 0.0) * dt;
    }
}
