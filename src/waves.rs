use std::ops::RangeInclusive;

use bevy::prelude::*;

use crate::{GameState, GameSystems};
/// Waves
/// 30 minute timer
/// Every 5 minutes, introduce new enemy type, increase # of enemies spawned every minute.
pub struct WavesPlugin;

impl Plugin for WavesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WaveTimer(0.0))
            .add_systems(OnEnter(GameState::Menu), reset_timer)
            .add_systems(Update, increment_timer.run_if(in_state(GameState::Playing)).after(GameSystems::Movement));
    }
}

#[derive(Resource)]
pub struct WaveTimer(pub f32);

fn increment_timer(mut commands: Commands, mut wave_timer: ResMut<WaveTimer>, time: Res<Time>) {
    let start_time = wave_timer.0;
    wave_timer.0 += time.delta_seconds();
    let new_time = wave_timer.0;
    WAVES.iter().skip_while(|w| w.run_at_secs < start_time).take_while(|w| w.run_at_secs < new_time).for_each(|value| {
        value.event.spawn(&mut commands);
    });
    
}

fn reset_timer(mut timer: ResMut<WaveTimer>) {
    timer.0 = 0.0;
}

const WAVES: &[WaveTimelineMarker] = &[WaveTimelineMarker { run_at_secs: 0.0, event: WaveTimelineEvent::SpawnEnemies(SpawnInfo { t: EnemyType::Red, count: 10, location: SpawnLocation::AroundPlayer { distance_range: 100.0..=1000.0 } }) }];

#[derive(Clone, Copy)]
enum EnemyType {
    Red
}


#[derive(Clone)]
enum SpawnLocation {
    /// Spawns enemies randomly around the player within the given range
    AroundPlayer {
        distance_range: RangeInclusive<f32>,
    },
}

#[derive(Event)]
struct SpawnAroundPlayerEvent {
    t: EnemyType,
    count: u32,
    range: RangeInclusive<f32>
}

#[derive(Clone)]
struct SpawnInfo {
    /// Type of enemy to spawn
    t: EnemyType,
    /// Number of enemies to spawn
    count: u32,
    /// Where to spawn enemies
    location: SpawnLocation
}

impl SpawnInfo {
    fn create_events(&self, commands: EventWriter<Spawn>) {
        let Self { t, count, location } = self.clone();
        match location {
            SpawnLocation::AroundPlayer { distance_range } => {
                
                SpawnAroundPlayerEvent { t, count, range: distance_range };
            },
        }
    }
}

struct WaveTimelineMarker {
    /// Time in seconds to run this marker
    run_at_secs: f32,
    event: WaveTimelineEvent
}

enum WaveTimelineEvent {
    SpawnEnemies(SpawnInfo),
}

impl WaveTimelineEvent {
    fn spawn(&self, commands: &mut Commands) {
        match self {
            WaveTimelineEvent::SpawnEnemies(enemies) => {
                enemies.create_events(commands);
            },
        }
    }
}
