use bevy::prelude::*;

use crate::{
    enemy::{Enemy, MoveToTarget},
    health::DeathEvent,
    loading::TextureAssets,
    player::Player,
    weapon::{ConstantAcceleration, Target, TargetVector, Velocity},
    GameState,
};

pub(crate) struct LevelSystemPlugin;

impl Plugin for LevelSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (award_player_xp, run_level_ups, xp_collisions).run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component)]
pub struct Xp(pub i32);

#[derive(Component)]
pub struct Level(pub i32);

/// How much XP should be awarded when this entity dies
#[derive(Component, Clone)]
pub struct XpWorth(pub i32);

#[derive(Component)]
struct XpPellet(XpWorth);

fn award_player_xp(
    mut commands: Commands,
    mut death_events: EventReader<DeathEvent>,
    enemies: Query<(&Transform, &XpWorth), With<Enemy>>,
    player: Query<Entity, With<Player>>,
    textures: Res<TextureAssets>,
) {
    let Ok(player) = player.get_single() else {
        return;
    };
    for events in death_events.read() {
        if let Ok((transform, xp)) = enemies.get(events.0) {
            commands.spawn((
                SpriteBundle {
                    texture: textures.xp_orb.clone(),
                    transform: Transform::from_translation(transform.translation),
                    sprite: Sprite {
                        custom_size: Some(Vec2::splat(25.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                XpPellet(xp.clone()),
                Velocity(Vec2::new(0.0, 100.0)),
                ConstantAcceleration(1.0),
                MoveToTarget,
                Target(Some(player)),
                TargetVector(None),
            ));
        }
    }
}

fn xp_collisions(
    mut commands: Commands,
    mut pellets: Query<(Entity, &Transform, &mut XpPellet)>,
    mut player: Query<(&Transform, &mut Xp), With<Player>>,
) {
    let Ok((player_transform, mut player_xp)) = player.get_single_mut() else {
        return;
    };
    for (entity, pellet_transform, mut xp) in pellets.iter_mut() {
        if (player_transform.translation - pellet_transform.translation).length_squared() < 1000.0 {
            player_xp.0 += xp.0 .0;
            xp.0 .0 = 0;
            commands.entity(entity).despawn();
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
            info!(
                "Leveled up! xp: {} level: {}. needed xp: {}",
                xp.0, level.0, needed_xp.0
            );
            next_state.set(GameState::Chooser);
        }
    }
}

#[inline]
pub(crate) fn xp_required_for_level(level: &Level) -> Xp {
    let xp_required = 71.429 * (level.0 as f32).powf(2.0) + 190.0;
    Xp(xp_required as i32)
}
