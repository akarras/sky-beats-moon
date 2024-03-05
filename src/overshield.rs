use bevy::prelude::*;

use crate::{health::Dead, GameState};

pub struct OvershieldPlugin;

impl Plugin for OvershieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, reset_overshield_state).add_systems(
            Update,
            recharge_shields.run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component)]
pub struct Overshield(pub u8);

impl Overshield {
    pub fn new(level: u8) -> Self {
        Self(level)
    }
}

#[derive(Component)]
pub struct OvershieldState {
    pub max_overshield: i32,
    pub current_overshield: i32,
    pub secs_until_recharge: f32,
}

/// Inserts a new overshield state anytime the overshield gets updated
fn reset_overshield_state(
    mut commands: Commands,
    overshield: Query<(Entity, &Overshield), Changed<Overshield>>,
) {
    for (entity, overshield) in overshield.iter() {
        let level = overshield.0;
        commands.entity(entity).insert(OvershieldState {
            max_overshield: level as i32 * 100,
            current_overshield: level as i32 * 100,
            secs_until_recharge: 0.0,
        });
    }
}

fn recharge_shields(mut overshield: Query<&mut OvershieldState, Without<Dead>>, time: Res<Time>) {
    for mut overshield in overshield.iter_mut() {
        if overshield.max_overshield > overshield.current_overshield {
            overshield.secs_until_recharge -= time.delta_seconds();
            if overshield.secs_until_recharge <= 0.0 {
                overshield.current_overshield += 1;
                overshield.secs_until_recharge = 0.1;
            }
        }
    }
}
