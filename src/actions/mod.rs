use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

use crate::actions::game_control::{get_movement, GameControl};
use crate::player::Player;
use crate::{GameState, GameSystems};

mod game_control;

pub const FOLLOW_EPSILON: f32 = 5.;

pub struct ActionsPlugin;

// This plugin listens for keyboard input and converts the input into Actions
// Actions can then be used as a resource in other systems to act on the player input.
impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Actions>().add_systems(
            Update,
            set_movement_actions
                .run_if(in_state(GameState::Playing))
                .in_set(GameSystems::Input),
        );
    }
}

#[derive(Default, Resource)]
pub struct Actions {
    pub player_movement: Option<Vec2>,
    pub camera_zoom: Option<f32>,
}

pub fn set_movement_actions(
    mut actions: ResMut<Actions>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    touch_input: Res<Touches>,
    player: Query<&Transform, With<Player>>,
    mut mouse_scroll: EventReader<MouseWheel>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    let mut player_movement = Vec2::new(
        get_movement(GameControl::Right, &keyboard_input)
            - get_movement(GameControl::Left, &keyboard_input),
        get_movement(GameControl::Up, &keyboard_input)
            - get_movement(GameControl::Down, &keyboard_input),
    );
    let mut zoom = None;
    // if there are two touches, see if they are pushing together or apart to generate the scrolling effect
    if touch_input.iter().count() == 2 {
        info!("Two touch!");
        let mut touches = touch_input.iter();
        let touch_1 = touches.next().unwrap();
        let touch_2 = touches.next().unwrap();
        let current_distance = touch_1.position() - touch_2.position();
        let previous_distance = touch_1.previous_position() - touch_2.previous_position();
        let diff = current_distance.length() - previous_distance.length();
        info!("Diff {diff}");
        if diff.abs() > 0.01 {
            zoom = Some(diff / 100.0);
        }
    } else {
        if let Some(touch_position) = touch_input.first_pressed_position() {
            let (camera, camera_transform) = camera.single();
            if let Some(touch_position) =
                camera.viewport_to_world_2d(camera_transform, touch_position)
            {
                let diff = touch_position - player.single().translation.xy();
                if diff.length() > FOLLOW_EPSILON {
                    player_movement = diff.normalize();
                }
            }
        }
    }
    for scroll in mouse_scroll.read() {
        let value = zoom.get_or_insert(0.0);
        *value -= scroll.y;
    }
    actions.camera_zoom = zoom;

    if player_movement != Vec2::ZERO {
        actions.player_movement = Some(player_movement.normalize());
    } else {
        actions.player_movement = None;
    }
}
