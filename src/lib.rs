#![allow(clippy::type_complexity)]

mod actions;
mod audio;
pub(crate) mod background_image;
mod clouds;
mod end_game;
pub(crate) mod enemy;
pub(crate) mod follow_camera;
pub(crate) mod health;
mod hud;
pub(crate) mod leveling;
mod loading;
mod menu;
pub(crate) mod overshield;
pub(crate) mod pause_menu;
mod player;
pub(crate) mod power_ups;
mod stats;
mod waves;
pub(crate) mod weapon;

use crate::actions::ActionsPlugin;
use crate::audio::InternalAudioPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::player::PlayerPlugin;

use background_image::BackgroundPlugin;
#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_rand::{plugin::EntropyPlugin, prelude::WyRand};
use clouds::CloudPlugin;
use end_game::EndGamePlugin;
use enemy::EnemyPlugin;
use follow_camera::FollowCameraPlugin;
use health::HealthPlugin;
use hud::HudPlugin;
use leveling::LevelSystemPlugin;
use overshield::OvershieldPlugin;
use pause_menu::PausePlugin;
use power_ups::PowerupPlugin;
use stats::StatsPlugin;
use waves::WavesPlugin;
use weapon::WeaponPlugin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub(crate) enum GameSystems {
    Input,
    /// Applies accelerations and changes to velocity & direct rotations
    PreMovement,
    /// Applies velocity and changes to rotation
    Movement,
    /// Checks for any collisions
    Collision,
    /// Draws UI elements on screen
    Ui,
}

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    Loading,
    // During this State the actual game logic is executed
    Playing,
    // Here the menu is drawn and waiting for player interaction
    Menu,
    /// The game is paused
    Paused,
    /// The player is choosing an item
    Chooser,
    /// End game screen shows stats from the round
    EndGame,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_plugins((
                EntropyPlugin::<WyRand>::default(),
                LoadingPlugin,
                MenuPlugin,
                ActionsPlugin,
                InternalAudioPlugin,
                PlayerPlugin,
                EnemyPlugin,
                FollowCameraPlugin,
                HealthPlugin,
                WeaponPlugin,
                PausePlugin,
                WavesPlugin,
                PowerupPlugin,
                CloudPlugin,
            ))
            .add_plugins((
                BackgroundPlugin,
                OvershieldPlugin,
                HudPlugin,
                StatsPlugin,
                EndGamePlugin,
                LevelSystemPlugin,
            ));

        #[cfg(debug_assertions)]
        {
            app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
        }
    }
}
