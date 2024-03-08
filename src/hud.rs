use bevy::prelude::*;

use crate::{
    health::{Health, MaxHealth},
    leveling::{xp_required_for_level, Level, Xp},
    overshield::OvershieldState,
    player::Player,
    stats::{EnemiesStillAlive, TotalEnemiesKilled},
    waves::WaveTimer,
    GameState, GameSystems,
};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), add_hud)
            .add_systems(OnExit(GameState::Playing), remove_hud)
            .add_systems(
                Update,
                (
                    update_shield_bar,
                    update_health_bar,
                    update_time_text,
                    update_enemy_counter_text,
                    update_xp_bar,
                )
                    .in_set(GameSystems::Ui)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component)]
struct Hud;

#[derive(Component)]
struct HealthBar;

#[derive(Component)]
struct ShieldBar;

#[derive(Component)]
struct EnemyCounter;

#[derive(Component)]
struct XpBar;

fn add_hud(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                align_items: AlignItems::Start,
                justify_items: JustifyItems::Start,
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|children| {
            children.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Vw(0.0),
                        height: Val::Vh(1.0),
                        ..Default::default()
                    },
                    background_color: BackgroundColor(Color::Rgba {
                        red: 0.7,
                        green: 0.1,
                        blue: 0.7,
                        alpha: 1.0,
                    }),
                    ..Default::default()
                },
                XpBar,
            ));
            children.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Vw(50.0),
                        height: Val::Vh(1.0),
                        ..Default::default()
                    },
                    background_color: BackgroundColor(Color::Rgba {
                        red: 1.0,
                        green: 0.1,
                        blue: 0.1,
                        alpha: 1.0,
                    }),
                    ..Default::default()
                },
                HealthBar,
            ));
            children.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Vw(0.0),
                        height: Val::Vh(1.0),
                        ..Default::default()
                    },
                    background_color: BackgroundColor(Color::Rgba {
                        red: 0.1,
                        green: 0.1,
                        blue: 1.0,
                        alpha: 1.0,
                    }),
                    ..Default::default()
                },
                ShieldBar,
            ));
            children
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Stretch,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|children| {
                    children.spawn((
                        TextBundle::from_section(
                            "0 alive",
                            TextStyle {
                                font_size: 30.0,
                                ..Default::default()
                            },
                        ),
                        EnemyCounter,
                    ));
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
        });
}

fn update_health_bar(
    mut healthbar: Query<&mut Style, With<HealthBar>>,
    player_health: Query<
        (&Health, &MaxHealth),
        (Or<(Changed<Health>, Changed<MaxHealth>)>, With<Player>),
    >,
) {
    for mut healthbar in healthbar.iter_mut() {
        for (health, max) in player_health.iter() {
            healthbar.width = Val::Vw(health.0 as f32 / max.0 as f32 * 100.0);
        }
    }
}

fn update_shield_bar(
    mut shield_bar: Query<&mut Style, With<ShieldBar>>,
    shield: Query<&OvershieldState, Changed<OvershieldState>>,
) {
    for mut shieldbar in shield_bar.iter_mut() {
        for shield in shield.iter() {
            shieldbar.width =
                Val::Vw(shield.current_overshield as f32 / shield.max_overshield as f32 * 100.0);
        }
    }
}

fn update_xp_bar(
    mut xp_bar: Query<&mut Style, With<XpBar>>,
    xp: Query<(&Xp, &Level), (Or<(Changed<Xp>, Changed<Level>)>, With<Player>)>,
) {
    for (xp, level) in xp.iter() {
        for mut bar in xp_bar.iter_mut() {
            let required_xp = xp_required_for_level(level);
            let width = (xp.0 as f32 / required_xp.0 as f32) * 100.0;
            info!("XP bar: {}/{} {}", xp.0, required_xp.0, width);
            bar.width = Val::Vw(width);
        }
    }
}

#[derive(Component)]
struct TimerText;

fn update_time_text(mut time: Query<&mut Text, With<TimerText>>, timer: Res<WaveTimer>) {
    for mut text in time.iter_mut() {
        let seconds = timer.0 as i32 % 60;
        let minutes = timer.0 as i32 / 60;
        *text = Text::from_section(
            format!("{minutes:0>2}:{seconds:0>2}"),
            TextStyle {
                font_size: 30.0,
                ..Default::default()
            },
        )
    }
}

fn update_enemy_counter_text(
    mut enemy: Query<&mut Text, With<EnemyCounter>>,
    alive: Res<EnemiesStillAlive>,
    killed: Res<TotalEnemiesKilled>,
) {
    for mut text in enemy.iter_mut() {
        *text = Text::from_section(
            format!("{} alive {} killed", alive.0, killed.0),
            TextStyle {
                font_size: 30.0,
                ..Default::default()
            },
        );
    }
}

fn remove_hud(mut commands: Commands, healthbar: Query<Entity, With<Hud>>) {
    for healthbar in healthbar.iter() {
        commands.entity(healthbar).despawn_recursive();
    }
}
