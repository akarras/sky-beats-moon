use bevy::prelude::*;

use crate::GameState;

pub struct PausePlugin;

#[derive(Component)]
struct PauseMenu;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, pause_menu_key.run_if(in_state(GameState::Playing)))
            .add_systems(
                Update,
                (unpause_menu_key, handle_main_menu_clicks).run_if(in_state(GameState::Paused)),
            )
            .add_systems(OnEnter(GameState::Paused), add_pause_menu)
            .add_systems(OnExit(GameState::Paused), cleanup_pause_menu);
    }
}

fn add_pause_menu(mut commands: Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            PauseMenu,
        ))
        .with_children(|children| {
            children.spawn(TextBundle::from_section(
                "Paused",
                TextStyle {
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                    ..Default::default()
                },
            ));
            children
                .spawn((
                    ButtonBundle {
                        background_color: BackgroundColor(Color::RED),
                        border_color: BorderColor(Color::DARK_GRAY),
                        ..Default::default()
                    },
                    MainMenuButton,
                ))
                .with_children(|c| {
                    c.spawn((TextBundle::from_section(
                        "Main Menu",
                        TextStyle {
                            font_size: 40.0,
                            ..Default::default()
                        },
                    ),));
                });
        });
}

#[derive(Component)]
pub struct MainMenuButton;

fn handle_main_menu_clicks(
    interactions: Query<&Interaction, (With<MainMenuButton>, Changed<Interaction>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in interactions.iter() {
        match interaction {
            Interaction::Pressed => next_state.set(GameState::Menu),
            _ => {}
        }
    }
}

fn cleanup_pause_menu(mut commands: Commands, pause_menu: Query<Entity, With<PauseMenu>>) {
    for entity in pause_menu.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn pause_menu_key(keys: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::BrowserBack) {
        next_state.set(GameState::Paused);
    }
}

fn unpause_menu_key(keys: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::BrowserBack) {
        next_state.set(GameState::Playing);
    }
}
