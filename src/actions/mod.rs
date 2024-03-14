use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

use crate::actions::game_control::{get_movement, GameControl};
use crate::{GameState, GameSystems};

mod game_control;

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
    pub touch_detected: bool,
}

pub fn set_movement_actions(
    mut actions: ResMut<Actions>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    touch_input: Res<Touches>,
    mut mouse_scroll: EventReader<MouseWheel>,
    camera: Query<&Camera, With<Camera2d>>,
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
            zoom = Some(-diff / 100.0);
        }
    } else {
        if let Some(touch_position) = touch_input.first_pressed_position() {
            actions.touch_detected = true;
            let camera = camera.single();
            if let Some(viewport_size) = camera.logical_viewport_size() {
                let joystick_position = Vec2::new(viewport_size.x / 2.0, viewport_size.y - 100.0);
                let joystick_offset = touch_position - joystick_position;
                player_movement = joystick_offset.normalize();
                player_movement.y = -player_movement.y;
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
