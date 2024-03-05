use bevy::prelude::*;

use crate::GameState;
/// Waves
/// 30 minute timer
/// Every 5 minutes, introduce new enemy type, increase # of enemies spawned every minute.
pub struct WavesPlugin;

impl Plugin for WavesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WaveTimer(0.0))
            .add_systems(Update, increment_timer.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Resource)]
pub struct WaveTimer(pub f32);

fn increment_timer(mut wave_timer: ResMut<WaveTimer>, time: Res<Time>) {
    wave_timer.0 += time.delta_seconds();
}
