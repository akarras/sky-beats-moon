use bevy::prelude::*;

use crate::{
    enemy::{EnemyType, Spawner},
    GameState, GameSystems,
};
/// Waves
/// 30 minute timer
/// Every 5 minutes, introduce new enemy type, increase # of enemies spawned every minute.
pub struct WavesPlugin;

impl Plugin for WavesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WaveTimer::default())
            .add_systems(OnEnter(GameState::Menu), reset_timer)
            .add_systems(
                Update,
                increment_timer
                    .run_if(in_state(GameState::Playing))
                    .after(GameSystems::Movement),
            );
    }
}

#[derive(Resource, Default)]
pub struct WaveTimer {
    pub current_time: f32,
    search_index: usize,
}

fn increment_timer(mut commands: Commands, mut wave_timer: ResMut<WaveTimer>, time: Res<Time>) {
    wave_timer.current_time += time.delta_seconds();
    let new_time = wave_timer.current_time;
    let Some(waves) = WAVES.as_ref().get(wave_timer.search_index..) else {
        return;
    };
    for wave in waves {
        if wave.run_at_secs < new_time {
            match &wave.event {
                WaveTimelineEvent::SpawnEnemies(spawner) => {
                    commands.spawn(spawner.clone());
                }
            }
            wave_timer.search_index += 1;
        } else {
            break;
        }
    }
}

fn reset_timer(mut timer: ResMut<WaveTimer>) {
    *timer = Default::default();
}

const WAVES: &[WaveTimelineMarker] = &[
    WaveTimelineMarker {
        run_at_secs: 0.0,
        event: WaveTimelineEvent::SpawnEnemies(Spawner {
            spawn_range: 1000.0..=2000.0,
            enemies_spawned_per_interval: 2,
            num_enemies: 1000,
            interval: 2.0,
            current_interval: 0.0,
            enemy_type: EnemyType::RedPlane,
        }),
    },
    WaveTimelineMarker {
        run_at_secs: 0.0,
        event: WaveTimelineEvent::SpawnEnemies(Spawner {
            spawn_range: 500.0..=500.0,
            enemies_spawned_per_interval: 10,
            num_enemies: 1000,
            interval: 1.0,
            current_interval: 1.0,
            enemy_type: EnemyType::Mosquito,
        }),
    },
];

struct WaveTimelineMarker {
    /// Time in seconds to run this marker
    run_at_secs: f32,
    event: WaveTimelineEvent,
}

enum WaveTimelineEvent {
    SpawnEnemies(Spawner),
}
