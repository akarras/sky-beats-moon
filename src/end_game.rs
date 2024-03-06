use bevy::prelude::*;

use crate::{stats::*, GameState};

struct EndGamePlugin;

impl Plugin for EndGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::EndGame), add_hud);
    }
}

fn add_hud(
    mut commands: Commands,
    enemies_killed: Res<TotalEnemiesKilled>,
    damage_done: Res<TotalDamageDone>,
    enemies_alive: Res<EnemiesStillAlive>,
) {
    let stats = [
        ("enemies killed", enemies_killed.0),
        ("damage done", damage_done.0),
        ("enemies alive", enemies_alive.0),
    ]
    .into_iter()
    .map(|(label, val)| {
        TextSection::new(
            format!("{label}: {val}"),
            TextStyle {
                font_size: 30.0,
                ..Default::default()
            },
        )
    });
    commands
        .spawn(NodeBundle {
            ..Default::default()
        })
        .with_children(|children| {
            children.spawn(TextBundle::from_sections(stats));
        });
}
