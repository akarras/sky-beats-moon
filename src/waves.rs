use bevy::prelude::*;

use crate::GameState;
/// Waves
/// 30 minute timer
/// Every 5 minutes, introduce new enemy type, increase # of enemies spawned every minute.
pub struct WavesPlugin;

impl Plugin for WavesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WaveTimer(0.0))
            .add_systems(
                Update,
                (increment_timer, update_time_text).run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                OnTransition {
                    from: GameState::Menu,
                    to: GameState::Playing,
                },
                add_timer_ui,
            )
            .add_systems(
                OnTransition {
                    from: GameState::Playing,
                    to: GameState::Menu,
                },
                remove_timer_ui,
            );
    }
}

#[derive(Resource)]
struct WaveTimer(f32);

fn increment_timer(mut wave_timer: ResMut<WaveTimer>, time: Res<Time>) {
    wave_timer.0 += time.delta_seconds();
}

#[derive(Component)]
struct TimerUi;

#[derive(Component)]
struct TimerText;

fn update_time_text(mut time: Query<&mut Text, With<TimerText>>, timer: Res<WaveTimer>) {
    for mut text in time.iter_mut() {
        let seconds = timer.0 as i32 % 60;
        let minutes = timer.0 as i32 / 60;
        *text = Text::from_section(
            format!("{minutes:2}:{seconds:2}"),
            TextStyle {
                font_size: 30.0,
                ..Default::default()
            },
        )
    }
}

fn add_timer_ui(mut commands: Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    justify_content: JustifyContent::End,
                    align_items: AlignItems::End,
                    justify_self: JustifySelf::End,
                    ..Default::default()
                },
                ..Default::default()
            },
            TimerUi,
        ))
        .with_children(|children| {
            children.spawn((
                TextBundle::from_section(
                    "0:00",
                    TextStyle {
                        font_size: 30.0,
                        ..Default::default()
                    },
                ),
                TimerText,
            ));
        });
}

fn remove_timer_ui(mut commands: Commands, timer: Query<Entity, With<TimerUi>>) {
    for entity in timer.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
