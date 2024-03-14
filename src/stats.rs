use bevy::prelude::*;

use crate::{
    enemy::Enemy,
    health::{DamageEvent, DeathEvent},
    GameState, GameSystems,
};
pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TotalDamageDone(0))
            // .insert_resource(TotalHealthLost(0))
            .insert_resource(TotalEnemiesKilled(0))
            .insert_resource(TotalBulletsFired(0))
            .insert_resource(EnemiesStillAlive(0))
            .add_systems(
                FixedUpdate,
                (count_enemies, collect_damage_done, count_deaths)
                    .after(GameSystems::Collision)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnEnter(GameState::Menu), reset_stats);
    }
}

fn reset_stats(
    mut total_damage_done: ResMut<TotalDamageDone>,
    mut enemies_killed: ResMut<TotalEnemiesKilled>,
    mut enemies_alive: ResMut<EnemiesStillAlive>,
    mut bullets_fired: ResMut<TotalBulletsFired>,
) {
    total_damage_done.0 = 0;
    enemies_alive.0 = 0;
    enemies_killed.0 = 0;
    bullets_fired.0 = 0;
}

#[derive(Resource)]
pub struct TotalDamageDone(pub u32);

fn collect_damage_done(
    mut damage_events: EventReader<DamageEvent>,
    mut damage_done: ResMut<TotalDamageDone>,
) {
    for damage in damage_events.read() {
        damage_done.0 += damage.amount as u32;
    }
}

// #[derive(Resource)]
// struct TotalHealthLost(u32);

#[derive(Resource)]
pub struct TotalEnemiesKilled(pub u32);

fn count_deaths(
    mut deaths: EventReader<DeathEvent>,
    enemies: Query<&Enemy>,
    mut enemies_killed: ResMut<TotalEnemiesKilled>,
) {
    for death in deaths.read() {
        if let Ok(_enemy) = enemies.get(death.0) {
            enemies_killed.0 += 1;
        }
    }
}

#[derive(Resource)]
pub struct TotalBulletsFired(pub u32);

#[derive(Resource)]
pub struct EnemiesStillAlive(pub u32);

fn count_enemies(enemies: Query<&Enemy>, mut accumulator: ResMut<EnemiesStillAlive>) {
    accumulator.0 = enemies.iter().len() as u32;
}
