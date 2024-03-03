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

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DamageEvent>()
            .add_event::<DeathEvent>()
            .add_systems(
                FixedPostUpdate,
                (check_dead, despawn, apply_damage).run_if(in_state(GameState::Playing)),
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
    for (mut dead, entity) in &mut despawners {
        dead.0 -= time.delta_seconds();
        if dead.0 <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn check_dead(
    mut commands: Commands,
    healths: Query<(&Health, Entity), Without<DespawnTimer>>,
    mut deaths: EventWriter<DeathEvent>,
) {
    for (health, entity) in &healths {
        if health.0 <= 0 {
            deaths.send(DeathEvent(entity));
            commands
                .entity(entity)
                .remove::<Health>()
                .try_insert(DespawnTimer(10.0));
        }
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
