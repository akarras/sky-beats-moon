use bevy::prelude::*;

use crate::{enemy::Enemy, health::DeathEvent, player::Player, GameState};

pub(crate) struct LevelSystemPlugin;

impl Plugin for LevelSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (award_player_xp, run_level_ups)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component)]
pub struct Xp(pub i32);

#[derive(Component)]
pub struct Level(pub i32);

/// How much XP should be awarded when this entity dies
#[derive(Component)]
pub struct XpWorth(pub i32);

fn award_player_xp(
    mut player: Query<&mut Xp, With<Player>>,
    mut death_events: EventReader<DeathEvent>,
    enemies: Query<&XpWorth, With<Enemy>>,
) {
    for events in death_events.read() {
        if let Ok(xp_value) = enemies.get(events.0) {
            if let Ok(mut player) = player.get_single_mut() {
                info!("Gained {}xp", xp_value.0);
                player.0 += xp_value.0;
            }
        }
    }
}

fn run_level_ups(
    mut player: Query<(&mut Level, &mut Xp), With<Player>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (mut level, mut xp) in player.iter_mut() {
        let needed_xp = xp_required_for_level(&level);
        if xp.0 >= needed_xp.0 {
            level.0 += 1;
            xp.0 -= needed_xp.0;
            info!("Leveled up! xp: {} level: {}. needed xp: {}", xp.0, level.0, needed_xp.0);
            next_state.set(GameState::Chooser);
        }
    }
}

pub fn xp_required_for_level(level: &Level) -> Xp {
    let xp_required = level.0.saturating_pow(2);
    Xp(xp_required)
}
