use std::ops::Deref;

use bevy::prelude::*;

use crate::{
    enemy::Enemy,
    health::{DamageEvent, Dead, DespawnTimer, Health},
    loading::TextureAssets,
    player::{OrientTowardsVelocity, Player},
    GameState,
};

pub struct WeaponPlugin;

#[derive(Component)]
pub struct Friendly;

#[derive(Component)]
pub struct Hostile;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_target_vectors,
                update_acceleration,
                apply_velocity,
                check_bullet_collisions_teamed::<Hostile, Friendly>,
                check_bullet_collisions_teamed::<Friendly, Hostile>,
                update_player_target,
                shoot_basic_gun::<MachineGun>,
                shoot_basic_gun::<PeaShooter>,
                shoot_basic_gun::<Sniper>,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

/// [Weapon] should provide the state for whether the weapon should be firing or not
#[derive(Component)]
pub struct Weapon {
    /// the base cooldown of the weapon
    pub cooldown: f32,
    /// the time left until the weapon fires again
    pub cooldown_left: f32,
}

#[derive(Component)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct ConstantAcceleration(f32);

#[derive(Component)]
pub struct Projectile {
    fired_by: Entity,
    damage_amount: i32,
    /// size of the projectile
    size: f32,
}

/// The entity that is being targeted by a weapon
#[derive(Component)]
pub struct Target(pub Option<Entity>);

/// The current vector that the target is located at
#[derive(Component)]
pub struct TargetVector(pub Option<Vec2>);

/// just tries to target the closest enemy
fn update_player_target(
    mut player: Query<(&mut Target, &Transform), With<Player>>,
    enemies: Query<(Entity, &Transform), (With<Enemy>, Without<DespawnTimer>)>,
) {
    if let Ok((mut target, start)) = player.get_single_mut() {
        target.0 = enemies
            .iter()
            .map(|(entity, enemy)| {
                (
                    entity,
                    (*Coord2D::from(start.translation) - *Coord2D::from(enemy.translation))
                        .length(),
                )
            })
            .min_by_key(|(_, dist)| *dist as i32)
            .map(|(e, _)| e);
    }
}

fn update_acceleration(
    mut velocities: Query<(&mut Velocity, &ConstantAcceleration)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    velocities.par_iter_mut().for_each(|(mut velocity, accel)| {
        let vel = velocity.0;
        velocity.0 += vel * (dt * accel.0)
    });
}

fn apply_velocity(mut entities: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    let dt = time.delta_seconds();
    entities
        .par_iter_mut()
        .for_each(|(mut transform, velocity)| {
            let velocity_2d = velocity.0 * dt;
            transform.translation += Vec3::new(velocity_2d.x, velocity_2d.y, 0.0);
        });
}

pub struct Coord2D(Vec2);

impl Deref for Coord2D {
    type Target = Vec2;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<Vec2> for Coord2D {
    fn as_ref(&self) -> &Vec2 {
        &self.0
    }
}

impl From<Vec2> for Coord2D {
    fn from(value: Vec2) -> Self {
        Coord2D(value)
    }
}

impl From<Vec3> for Coord2D {
    fn from(value: Vec3) -> Self {
        Vec2::new(value.x, value.y).into()
    }
}

#[derive(Component)]
struct DeathParticles(Option<Box<dyn Fn(&mut Commands, Transform) + Send + Sync>>);

/// The entity that was last hit
#[derive(Component)]
struct LastHit(Entity);

/// Checks collisions between particles
fn check_bullet_collisions_teamed<A, B>(
    commands: ParallelCommands,
    mut bullets: Query<
        (
            Entity,
            &Transform,
            &Projectile,
            Option<&mut DeathParticles>,
            &mut Health,
            Option<&mut LastHit>,
        ),
        With<A>,
    >,
    other_entities: Query<(Entity, &Transform), (With<Health>, With<B>)>,
) where
    A: Component,
    B: Component,
{
    bullets.par_iter_mut().for_each(
        |(
            self_bullet,
            transform,
            Projectile {
                fired_by,
                damage_amount,
                size,
            },
            death_particles,
            mut health,
            last_hit,
        )| {
            for (entity, other_transform) in &other_entities {
                // don't collide with sender
                let fired_by = *fired_by;
                if self_bullet != entity
                    && fired_by != entity
                    && last_hit.as_ref().map(|l| l.0 != entity).unwrap_or(true)
                {
                    let delta = *Coord2D::from(transform.translation)
                        - *Coord2D::from(other_transform.translation);
                    if delta.length() < *size {
                        let amount = *damage_amount;
                        health.0 -= 1;

                        commands.command_scope(|mut cmds| {
                            if health.0 <= 0 {
                                cmds.entity(self_bullet).despawn();
                            } else if let Some(mut hit) = last_hit {
                                hit.0 = entity;
                            } else {
                                cmds.entity(self_bullet).try_insert(LastHit(entity));
                            }
                            if let Some(death_particles) =
                                death_particles.as_ref().and_then(|u| u.0.as_ref())
                            {
                                death_particles(&mut cmds, *transform);
                            }
                            cmds.add(move |w: &mut World| {
                                w.send_event(DamageEvent {
                                    damaged_by: fired_by,
                                    applied_to: entity,
                                    amount,
                                });
                            });
                        });
                        break;
                    }
                }
            }
        },
    );
}

fn update_target_vectors(
    mut weapons: Query<(Entity, &mut TargetVector, &Target), Without<DespawnTimer>>,
    transforms: Query<&Transform>,
) {
    weapons
        .par_iter_mut()
        .for_each(|(current_entity, mut vector, target)| {
            if let Some(target) = target.0 {
                if let Ok([source, target]) = transforms.get_many([current_entity, target]) {
                    let direction = target.translation - source.translation;
                    let direction_2d = Vec2::new(direction.x, direction.y);
                    vector.0 = (1000.0 * 1000.0 > direction_2d.length_squared())
                        .then(|| direction_2d.normalize());
                }
            }
        });
}

trait BasicGun {
    fn cooldown(&self) -> f32;
    fn damage(&self) -> i32;
    fn projectile_velocity(&self) -> f32;
    fn projectile_sprite(assets: &Res<TextureAssets>) -> Handle<Image>;
    fn cooldown_remaining(&mut self) -> &mut f32;
    fn health(&self) -> i32;
}

#[derive(Component)]
pub struct MachineGun {
    level: u8,
    cooldown_remaining: f32,
}

impl MachineGun {
    pub fn new(level: u8) -> Self {
        MachineGun {
            level: level,
            cooldown_remaining: 0.5,
        }
    }
}

impl BasicGun for MachineGun {
    fn cooldown(&self) -> f32 {
        match self.level {
            0 => 0.5,
            1 => 0.5,
            2 => 0.4,
            3 => 0.3,
            4 => 0.2,
            5 => 0.1,
            _ => 0.05,
        }
    }

    fn damage(&self) -> i32 {
        match self.level {
            0..=2 => 2,
            3..=5 => 5,
            _ => 6,
        }
    }

    fn projectile_velocity(&self) -> f32 {
        150.0
    }

    fn projectile_sprite(assets: &Res<TextureAssets>) -> Handle<Image> {
        assets.bullet.clone()
    }

    fn cooldown_remaining(&mut self) -> &mut f32 {
        &mut self.cooldown_remaining
    }

    fn health(&self) -> i32 {
        1
    }
}

#[derive(Component)]
pub struct PeaShooter {
    level: u8,
    cooldown_remaining: f32,
}

impl PeaShooter {
    pub fn new(level: u8) -> Self {
        PeaShooter {
            level: level,
            cooldown_remaining: 0.5,
        }
    }
}

impl BasicGun for PeaShooter {
    fn cooldown(&self) -> f32 {
        match self.level {
            0 => 2.5,
            1 => 1.0,
            2 => 0.9,
            3 => 0.8,
            4 => 0.7,
            5 => 0.6,
            _ => 0.5,
        }
    }

    fn damage(&self) -> i32 {
        match self.level {
            0..=2 => 5,
            3..=5 => 10,
            _ => 6,
        }
    }

    fn projectile_velocity(&self) -> f32 {
        200.0
    }

    fn projectile_sprite(assets: &Res<TextureAssets>) -> Handle<Image> {
        assets.pea.clone()
    }

    fn cooldown_remaining(&mut self) -> &mut f32 {
        &mut self.cooldown_remaining
    }

    fn health(&self) -> i32 {
        2
    }
}

#[derive(Component)]
pub struct Sniper {
    level: u8,
    cooldown_remaining: f32,
}

impl Sniper {
    pub fn new(level: u8) -> Self {
        Sniper {
            level: level,
            cooldown_remaining: 0.5,
        }
    }
}

impl BasicGun for Sniper {
    fn cooldown(&self) -> f32 {
        match self.level {
            0 => 5.0,
            1 => 5.0,
            2 => 5.0,
            3 => 5.0,
            4 => 4.0,
            5 => 3.0,
            _ => 2.0,
        }
    }

    fn damage(&self) -> i32 {
        match self.level {
            0..=2 => 100,
            3..=5 => 100,
            _ => 6,
        }
    }

    fn projectile_velocity(&self) -> f32 {
        500.0
    }

    fn health(&self) -> i32 {
        10
    }

    fn projectile_sprite(assets: &Res<TextureAssets>) -> Handle<Image> {
        assets.bullet.clone()
    }

    fn cooldown_remaining(&mut self) -> &mut f32 {
        &mut self.cooldown_remaining
    }
}

fn shoot_basic_gun<T>(
    commands: ParallelCommands,
    mut machine_gun: Query<
        (
            Entity,
            &TargetVector,
            &mut T,
            &Transform,
            Option<&SpecialMunitions>,
            Option<&Friendly>,
            Option<&Enemy>,
        ),
        Without<Dead>,
    >,
    textures: Res<TextureAssets>,
    time: Res<Time>,
) where
    T: Component + BasicGun + Sized,
{
    let dt = time.delta_seconds();
    machine_gun.par_iter_mut().for_each(
        |(fired_by, vector, mut gun, transform, munitions, friendly, enemy)| {
            if let Some(target_vector) = vector.0 {
                if *gun.cooldown_remaining() <= 0.0 {
                    *gun.cooldown_remaining() = gun.cooldown();
                    let death_texture = textures.bullet_impact.clone();
                    commands.command_scope(|mut cmd| {
                        let mut entity = cmd.spawn((
                            SpriteBundle {
                                texture: T::projectile_sprite(&textures),
                                transform: transform.with_scale(Vec3::splat(1.0)),
                                ..Default::default()
                            },
                            Projectile {
                                fired_by,
                                damage_amount: gun.damage()
                                    * munitions.map(|m| m.damage_mult()).unwrap_or(1),
                                size: 40.0,
                            },
                            Velocity(target_vector * gun.projectile_velocity()),
                            Health(gun.health()),
                            DeathParticles(Some(Box::new(move |cmds, transform| {
                                cmds.spawn((
                                    SpriteBundle {
                                        texture: death_texture.clone(),
                                        transform,
                                        ..Default::default()
                                    },
                                    DespawnTimer(0.2),
                                ));
                            }))),
                            DespawnTimer(30.0),
                            OrientTowardsVelocity,
                        ));
                        if friendly.is_some() {
                            entity.insert(Friendly);
                        }
                        if enemy.is_some() {
                            entity.insert(Hostile);
                        }
                    });
                } else {
                    *gun.cooldown_remaining() -= dt;
                }
            }
        },
    );
}

#[derive(Component)]
pub struct SpecialMunitions(u8);

impl SpecialMunitions {
    pub fn new(level: u8) -> Self {
        Self(level)
    }

    fn damage_mult(&self) -> i32 {
        self.0 as i32
    }
}
