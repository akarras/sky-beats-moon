use bevy::prelude::*;

use crate::{stats::*, GameState};

pub struct EndGamePlugin;

impl Plugin for EndGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::EndGame), add_death_screen)
            .add_systems(OnExit(GameState::EndGame), cleanup_death_screen)
            .add_systems(Update, main_menu_handler);
    }
}

#[derive(Component)]
struct MainMenuButton;

#[derive(Component)]
struct DeathScreen;

fn add_death_screen(
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
        .spawn((
            NodeBundle {
                style: Style {
                    height: Val::Vh(50.0),
                    width: Val::Vw(50.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_items: JustifyItems::Center,
                    ..Default::default()
                },
                ..Default::default()
            },
            DeathScreen,
        ))
        .with_children(|children| {
            children.spawn(TextBundle::from_sections(stats));
            children
                .spawn((
                    ButtonBundle {
                        background_color: BackgroundColor(Color::DARK_GREEN),
                        ..Default::default()
                    },
                    MainMenuButton,
                ))
                .with_children(|btn| {
                    btn.spawn((TextBundle::from_section(
                        "Main Menu",
                        TextStyle {
                            font_size: 30.0,
                            ..Default::default()
                        },
                    ),));
                });
        });
}

fn cleanup_death_screen(mut commands: Commands, death_screen: Query<Entity, With<DeathScreen>>) {
    for death in death_screen.iter() {
        commands.entity(death).despawn_recursive();
    }
}

fn main_menu_handler(
    menu_button: Query<&Interaction, (With<MainMenuButton>, Changed<Interaction>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for button in menu_button.iter() {
        match button {
            Interaction::Pressed => {
                next_state.set(GameState::Menu);
            }
            _ => {}
        }
    }
}
