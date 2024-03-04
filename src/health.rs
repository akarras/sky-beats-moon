use bevy::prelude::*;

use crate::GameState;

pub struct HealthPlugin;

#[derive(Component)]
pub struct Health(pub i32);

#[derive(Component)]
pub struct MaxHealth(pub i32);

#[derive(Event)]
pub struct DamageEvent {
    pub damaged_by: Entity,
    pub applied_to: Entity,
    pub amount: i32,
}

#[derive(Event)]
pub struct DeathEvent(pub Entity);

#[derive(Component)]
pub struct Dead;

#[derive(Component)]
pub struct DeadTexture(pub Handle<Image>);

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DamageEvent>()
            .add_event::<DeathEvent>()
            .add_systems(
                Update,
                (check_dead, despawn, apply_damage, apply_dead_texture)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component)]
pub struct DespawnTimer(pub f32);

fn despawn(
    mut commands: Commands,
    mut despawners: Query<(&mut DespawnTimer, Entity)>,
    time: Res<Time>,
) {
    for (mut despawn, entity) in &mut despawners {
        despawn.0 -= time.delta_seconds();
        if despawn.0 <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn check_dead(
    mut commands: Commands,
    healths: Query<(&Health, Entity), (Without<DespawnTimer>, Changed<Health>)>,
    mut deaths: EventWriter<DeathEvent>,
) {
    for (health, entity) in &healths {
        if health.0 <= 0 {
            deaths.send(DeathEvent(entity));
            commands
                .entity(entity)
                // .remove::<Health>()
                .try_insert(DespawnTimer(10.0))
                .insert(Dead);
        }
    }
}

fn apply_dead_texture(mut images: Query<(&mut Handle<Image>, &DeadTexture), Changed<Dead>>) {
    for (mut image, dead_texture) in images.iter_mut() {
        *image = dead_texture.0.clone();
    }
}

fn apply_damage(mut incoming_events: EventReader<DamageEvent>, mut health: Query<&mut Health>) {
    for DamageEvent {
        damaged_by,
        applied_to,
        amount,
    } in incoming_events.read()
    {
        if let Ok(mut target) = health.get_mut(*applied_to) {
            target.0 -= *amount;
            info!(
                "Damage done {:?}->{:?}: {}. Health remaining: {}",
                damaged_by, applied_to, *amount, target.0
            );
        }
    }
}
