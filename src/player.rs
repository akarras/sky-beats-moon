use crate::actions::Actions;
use crate::health::{DeadTexture, DespawnTimer, Health, MaxHealth};
use crate::leveling::{Level, Xp};
use crate::loading::TextureAssets;
use crate::power_ups::Powerups;
use crate::weapon::{Friendly, Target, TargetVector, Velocity, Weapon};
use crate::{GameState, GameSystems};
use bevy::prelude::*;

pub struct PlayerPlugin;

#[derive(Component)]
pub struct Player;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnTransition {
                from: GameState::Menu,
                to: GameState::Playing,
            },
            spawn_player,
        )
        .add_systems(OnEnter(GameState::Menu), despawn_player)
        .add_systems(
            Update,
            (
                (move_player, turn_to_match_velocity)
                    .in_set(GameSystems::PreMovement)
                    .after(GameSystems::Input),
                dead_player,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

fn spawn_player(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    commands
        .spawn((
            SpriteBundle {
                texture: textures.player.clone(),
                transform: Transform::from_translation(Vec3::new(0., 0., 1.))
                    .with_scale(Vec3::new(0.5, 0.5, 0.5)),
                ..Default::default()
            },
            Weapon {
                cooldown: 0.1,
                cooldown_left: 1.0,
            },
            Target(None),
            TargetVector(None),
            MaxHealth(100),
            Health(100),
            Velocity(Vec2::new(0.0, 100.0)),
            Friendly,
            Powerups::default(),
            OrientTowardsVelocity,
            DeadTexture(textures.player_dead.clone()),
            Xp(0),
            Level(1)
        ))
        .insert(Player);
    next_state.set(GameState::Chooser);
}

fn despawn_player(mut commands: Commands, player: Query<Entity, With<Player>>) {
    for player in player.iter() {
        commands.entity(player).despawn();
    }
}

fn move_player(
    time: Res<Time>,
    actions: Res<Actions>,
    mut player_query: Query<&mut Velocity, (With<Player>, Without<DespawnTimer>)>,
) {
    if actions.player_movement.is_none() {
        return;
    }
    let speed = 1000.;
    let movement = Vec2::new(
        actions.player_movement.unwrap().x * speed * time.delta_seconds(),
        actions.player_movement.unwrap().y * speed * time.delta_seconds(),
    );
    for mut player_velocity in &mut player_query {
        player_velocity.0 += movement;
        player_velocity.0 = player_velocity.0.clamp_length_max(200.0);
    }
}

#[derive(Component)]
pub struct OrientTowardsVelocity;

fn turn_to_match_velocity(
    mut query: Query<(&mut Transform, &Velocity), With<OrientTowardsVelocity>>,
) {
    query.par_iter_mut().for_each(|(mut transform, velocity)| {
        let direction = velocity.0.normalize();
        transform.rotation = Quat::from_rotation_arc(Vec3::Y, direction.extend(0.0));
    });
}

fn dead_player(
    mut dead_player: RemovedComponents<Player>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for _dead in dead_player.read() {
        next_state.set(GameState::EndGame);
    }
}
