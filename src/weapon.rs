use std::ops::Deref;

use bevy::prelude::*;

use crate::{
    enemy::Enemy,
    health::{DamageEvent, DespawnTimer, Health},
    player::Player,
    GameState,
};

pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_target_vectors,
                update_acceleration,
                apply_velocity,
                check_bullet_collisions,
                update_player_target,
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
pub struct TargetVector(Option<Vec2>);

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
    for (mut velocity, accel) in &mut velocities {
        let vel = velocity.0;
        velocity.0 += vel * (dt * accel.0)
    }
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

fn check_bullet_collisions(
    commands: ParallelCommands,
    bullets: Query<(Entity, &Transform, &Projectile)>,
    other_entities: Query<(Entity, &Transform), With<Health>>,
) {
    bullets.par_iter().for_each(
        |(
            self_bullet,
            transform,
            Projectile {
                fired_by,
                damage_amount,
                size,
            },
        )| {
            for (entity, other_transform) in &other_entities {
                // don't collide with sender
                let fired_by = *fired_by;
                if self_bullet != entity && fired_by != entity {
                    let delta = *Coord2D::from(transform.translation)
                        - *Coord2D::from(other_transform.translation);
                    if delta.length() < *size {
                        let amount = *damage_amount;
                        commands.command_scope(|mut cmds| {
                            cmds.entity(self_bullet).despawn();
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
