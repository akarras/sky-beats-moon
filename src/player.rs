use crate::actions::Actions;
use crate::health::{DespawnTimer, Health, MaxHealth};
use crate::loading::TextureAssets;
use crate::weapon::{Target, Velocity, Weapon};
use crate::GameState;
use bevy::prelude::*;

pub struct PlayerPlugin;

#[derive(Component)]
pub struct Player;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(Update, move_player.run_if(in_state(GameState::Playing)));
    }
}

fn spawn_player(mut commands: Commands, textures: Res<TextureAssets>) {
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
                range: 10000.0,
            },
            Target(None),
            MaxHealth(10000000),
            Health(10000000),
            Velocity(Vec2::new(0.0, 100.0)),
        ))
        .insert(Player);
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
