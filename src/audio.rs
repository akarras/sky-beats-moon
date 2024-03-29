use std::time::Duration;

use crate::actions::{set_movement_actions, Actions};
use crate::health::DespawnTimer;
use crate::loading::AudioAssets;
use crate::player::Player;
use crate::GameState;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

pub struct InternalAudioPlugin;

// This plugin is responsible to control the game audio
impl Plugin for InternalAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioPlugin)
            .add_systems(OnEnter(GameState::Playing), start_audio)
            .add_systems(
                Update,
                control_flying_sound
                    .after(set_movement_actions)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), stop_audio);
    }
}

#[derive(Resource)]
struct FlyingAudio(Handle<AudioInstance>);

fn start_audio(mut commands: Commands, audio_assets: Res<AudioAssets>, audio: Res<Audio>) {
    audio.pause();
    let handle = audio
        .play(audio_assets.flying.clone())
        .looped()
        .with_volume(0.3)
        .handle();
    commands.insert_resource(FlyingAudio(handle));
}

fn stop_audio(mut audio_instances: ResMut<Assets<AudioInstance>>) {
    for audio in audio_instances.iter_mut() {
        audio.1.stop(AudioTween::linear(Duration::from_millis(500)));
    }
}

fn control_flying_sound(
    actions: Res<Actions>,
    player_state: Query<&Player, Without<DespawnTimer>>,
    audio: Res<FlyingAudio>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    if let Some(instance) = audio_instances.get_mut(&audio.0) {
        match instance.state() {
            PlaybackState::Paused { .. } => {
                if actions.player_movement.is_some() && player_state.get_single().is_ok() {
                    instance.resume(AudioTween::default());
                }
            }
            PlaybackState::Playing { .. } => {
                if actions.player_movement.is_none() && player_state.get_single().is_err() {
                    instance.pause(AudioTween::default());
                }
            }
            _ => {}
        }
    }
}
