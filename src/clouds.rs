use bevy::prelude::*;
use bevy_rand::{prelude::WyRand, resource::GlobalEntropy};
use rand::{seq::SliceRandom, Rng};

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
        );
    }
}

fn spawn_clouds(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    mut rand: ResMut<GlobalEntropy<WyRand>>,
) {
    let spawn_range = 20000.0;
    let wind_speed = 200.0;
    let number_of_clouds = rand.gen_range(1..200);
    let mut rand = &mut *rand;
    let clouds = (0..number_of_clouds)
        .into_iter()
        .map(|_i| {
            let velocity = Vec2::new(
                rand.gen_range(-wind_speed..wind_speed),
                rand.gen_range(-wind_speed..wind_speed),
            );
            (
                SpriteBundle {
                    texture: (*[&textures.cloud_1, &textures.cloud_2]
                        .choose(&mut rand)
                        .unwrap())
                    .clone(),
                    transform: Transform::from_xyz(
                        rand.gen_range(-spawn_range..spawn_range),
                        rand.gen_range(-spawn_range..spawn_range),
                        -10.0,
                    )
                    .with_scale(Vec3::splat(rand.gen_range(0.5..4.0)))
                    .with_rotation(Quat::from_rotation_arc(Vec3::Y, velocity.extend(0.0))),
                    ..Default::default()
                },
                Cloud,
                Velocity(velocity),
            )
        })
        .collect::<Vec<_>>();
    commands.spawn_batch(clouds);
}
