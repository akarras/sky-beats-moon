use std::ops::RangeInclusive;

use bevy::prelude::*;
use bevy_rand::{prelude::WyRand, resource::GlobalEntropy};
use rand::Rng;

use crate::{
    health::{DeadTexture, DespawnTimer, Health, MaxHealth}, leveling::XpWorth, loading::TextureAssets, player::{OrientTowardsVelocity, Player}, power_ups::{PowerUpType, Powerup, Powerups}, weapon::{Coord2D, Hostile, Target, TargetVector, Velocity}, GameState
};

pub struct EnemyPlugin;

#[derive(Component)]
pub struct Enemy;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), create_spawner)
            .add_systems(OnEnter(GameState::Menu), cleanup_enemies)
            .add_systems(Update, move_enemies.run_if(in_state(GameState::Playing)))
            .add_systems(Update, spawn_enemies.run_if(in_state(GameState::Playing)));
    }
}

fn cleanup_enemies(mut commands: Commands, enemies: Query<Entity, With<Enemy>>) {
    for enemy in enemies.iter() {
        commands.entity(enemy).despawn();
    }
}

#[derive(Component)]
pub struct Spawner {
    respawn_rate: f64,
    spawn_range: RangeInclusive<f32>,
}

fn create_spawner(mut commands: Commands) {
    commands.spawn_batch([
        (Spawner {
            respawn_rate: 1.0,
            spawn_range: 1000.0..=6000.0,
        },),
        (Spawner {
            respawn_rate: 2.0,
            spawn_range: 1000.0..=10000.0,
        },),
        (Spawner {
            respawn_rate: 2.0,
            spawn_range: 1000.0..=100000.0,
        },),
    ]);
}

fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut spawner: Query<&mut Spawner>,
    textures: Res<TextureAssets>,
    player: Query<(Entity, &Transform), With<Player>>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
) {
    for mut spawner in spawner.iter_mut() {
        spawner.respawn_rate -= time.delta_seconds_f64();

        if spawner.respawn_rate < 0.0 {
            if let Ok(player) = player.get_single() {
                let player_location = player.1.translation;
                let theta = rng.gen_range::<f32, _>(-10000.0..10000.0);
                let x = theta.sin();
                let y = theta.cos();
                let distance = rng.gen_range(spawner.spawn_range.clone());
                let relative_position = Vec3::new(x, y, 0.0) * distance;
                let new_location =
                    *Coord2D::from(player_location) + *Coord2D::from(relative_position);
                spawner.respawn_rate = 1.0;

                // info!("Spawning enemy! {relative_position} {theta}");
                commands.spawn((
                    SpriteBundle {
                        texture: textures.red_plane.clone(),
                        transform: Transform::from_xyz(new_location.x, new_location.y, 1.0),
                        ..Default::default()
                    },
                    Enemy,
                    Hostile,
                    Velocity(Vec2::new(-x, -y)),
                    MaxHealth(2),
                    Health(2),
                    Powerups([
                        Some(Powerup {
                            power: PowerUpType::PeaShooter,
                            level: 0,
                        }),
                        None,
                        None,
                        None,
                        None,
                        None,
                    ]),
                    Target(Some(player.0)),
                    TargetVector(None),
                    OrientTowardsVelocity,
                    DeadTexture(textures.red_plane_dead.clone()),
                    XpWorth(10)
                ));
            }
        }
    }
}

fn move_enemies(
    mut current_enemies: Query<(&mut Velocity, &Transform), (With<Enemy>, Without<DespawnTimer>)>,
    player: Query<&Transform, (With<Player>, Without<Enemy>)>,
    time: Res<Time>,
) {
    let delta = time.delta_seconds();
    if let Ok(player_transform) = player.get_single() {
        current_enemies
            .par_iter_mut()
            .for_each(|(mut velocity, transform)| {
                let offset = player_transform.translation - transform.translation;
                if let Some(normalize) = Vec2::new(offset.x, offset.y).try_normalize() {
                    velocity.0 += normalize * 200.0 * delta;
                    velocity.0 = velocity.0.clamp_length_max(100.0);
                }
            });
    }
}
