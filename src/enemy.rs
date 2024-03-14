use std::ops::RangeInclusive;

use bevy::prelude::*;
use bevy_rand::{prelude::WyRand, resource::GlobalEntropy};
use rand::Rng;

use crate::{
    health::{DeadTexture, DespawnTimer, Health, MaxHealth},
    leveling::XpWorth,
    loading::TextureAssets,
    player::{OrientTowardsVelocity, Player},
    power_ups::{PowerUpType, Powerup, Powerups},
    weapon::{ConstantAcceleration, Coord2D, Hostile, Target, TargetVector, VMax, Velocity},
    GameState,
};
use torcurve_rs::torcurve;

pub struct EnemyPlugin;

#[derive(Component, Default)]
pub struct Enemy;

#[derive(Component)]
pub struct MoveToTarget;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), cleanup_enemies)
            .add_systems(
                Update,
                move_towards_target.run_if(in_state(GameState::Playing)),
            )
            .add_systems(Update, spawn_enemies.run_if(in_state(GameState::Playing)));
    }
}

fn cleanup_enemies(mut commands: Commands, enemies: Query<Entity, With<Enemy>>) {
    for enemy in enemies.iter() {
        commands.entity(enemy).despawn();
    }
}

#[derive(Copy, Clone)]
pub(crate) enum EnemyType {
    RedPlane,
    Mosquito,
    SailBoat,
}

#[derive(Component, Clone)]
pub struct Spawner {
    /// The distance from the player that the spawner will spawn enemies at
    pub(crate) spawn_range: RangeInclusive<f32>,
    pub(crate) interval: f32,
    pub(crate) current_interval: f32,
    /// How many enemies per interval will be spawned
    pub(crate) enemies_spawned_per_interval: u32,
    /// The remaining number of enemies this spawner will create
    pub(crate) num_enemies: i32,
    pub(crate) enemy_type: EnemyType,
}

#[derive(Bundle)]
struct HealthBundle {
    health: Health,
    max_health: MaxHealth,
}

impl HealthBundle {
    const fn new(health: i32) -> Self {
        HealthBundle {
            health: Health(health),
            max_health: MaxHealth(health),
        }
    }
}

#[derive(Bundle)]
struct RedPlaneBundle {
    sprite: SpriteBundle,
    tags: (Enemy, Hostile),
    health: HealthBundle,
    target: Target,
    velocity: Velocity,
    target_vector: TargetVector,
    powerups: Powerups,
    death_texture: DeadTexture,
    orient_toward_velocity: OrientTowardsVelocity,
    xp_worth: XpWorth,
    move_to_target: MoveToTarget,
    acceleration: ConstantAcceleration,
    vmax: VMax,
}

impl RedPlaneBundle {
    fn new(textures: &TextureAssets, location: Transform, velocity: Vec2, target: Entity) -> Self {
        Self {
            sprite: SpriteBundle {
                texture: textures.red_plane.clone(),
                transform: location,
                ..Default::default()
            },
            tags: (Enemy, Hostile),
            health: HealthBundle::new(20),
            target: Target(Some(target)),
            target_vector: TargetVector(None),
            powerups: Powerups([
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
            velocity: Velocity(velocity),
            death_texture: DeadTexture(textures.red_plane_dead.clone()),
            orient_toward_velocity: OrientTowardsVelocity,
            xp_worth: XpWorth(10),
            move_to_target: MoveToTarget,
            acceleration: ConstantAcceleration(1000.0),
            vmax: VMax(100.0),
        }
    }
}

#[derive(Bundle)]
struct MosquitoBundle {
    sprite: SpriteBundle,
    tags: (Enemy, Hostile),
    health: HealthBundle,
    target: Target,
    target_vector: TargetVector,
    orient_towards_velocity: OrientTowardsVelocity,
    xp_worth: XpWorth,
    move_to_target: MoveToTarget,
    acceleration: ConstantAcceleration,
    vmax: VMax,
    velocity: Velocity,
}

impl MosquitoBundle {
    fn new(assets: &TextureAssets, target: Entity, transform: Transform) -> Self {
        MosquitoBundle {
            sprite: SpriteBundle {
                texture: assets.bevy.clone(),
                transform,
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(25.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            tags: Default::default(),
            health: HealthBundle::new(2),
            target: Target(Some(target)),
            target_vector: TargetVector(None),
            orient_towards_velocity: OrientTowardsVelocity,
            xp_worth: XpWorth(1),
            move_to_target: MoveToTarget,
            acceleration: ConstantAcceleration(100.0),
            vmax: VMax(500.0),
            velocity: Velocity(Vec2::splat(0.01)),
        }
    }
}

fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut spawner: Query<(Entity, &mut Spawner)>,
    textures: Res<TextureAssets>,
    player: Query<(Entity, &Transform), With<Player>>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
) {
    'spawner: for (spawner_entity, mut spawner) in spawner.iter_mut() {
        spawner.current_interval -= time.delta_seconds();
        if spawner.current_interval < 0.0 {
            spawner.current_interval = spawner.interval;
            if let Ok(player) = player.get_single() {
                for _count in 0..spawner.enemies_spawned_per_interval {
                    if spawner.num_enemies <= 0 {
                        commands.entity(spawner_entity).despawn();
                        continue 'spawner;
                    }
                    let player_location = player.1.translation;
                    let theta = rng.gen_range::<f32, _>(-10000.0..10000.0);
                    let x = theta.sin();
                    let y = theta.cos();
                    let distance = rng.gen_range(spawner.spawn_range.clone());
                    let relative_position = Vec3::new(x, y, 0.0) * distance;
                    let new_location =
                        *Coord2D::from(player_location) + *Coord2D::from(relative_position);
                    let transform = Transform::from_translation(new_location.extend(1.0));
                    // info!("Spawning enemy! {relative_position} {theta}");
                    match spawner.enemy_type {
                        EnemyType::RedPlane => {
                            commands.spawn(RedPlaneBundle::new(
                                &textures,
                                transform,
                                Vec2::new(-x, -y),
                                player.0,
                            ));
                        }
                        EnemyType::Mosquito => {
                            commands.spawn(MosquitoBundle::new(&textures, player.0, transform));
                        }
                        EnemyType::SailBoat => {}
                    }

                    spawner.num_enemies -= 1;
                }
            }
        }
    }
}

fn move_towards_target(
    mut current_enemies: Query<
        (&mut Velocity, &TargetVector),
        (With<MoveToTarget>, Without<DespawnTimer>),
    >,
    time: Res<Time>,
) {
    let _delta = time.delta_seconds();
    current_enemies
        .par_iter_mut()
        .for_each(|(mut velocity, target_vector)| {
            let length = velocity.0.length();
            if let Some(target) = target_vector.0 {
                if let Some(vec) = target_vector.0 {
                    let target_rotation = vec * length;
                }
            }
        });
}
