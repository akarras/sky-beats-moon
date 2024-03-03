use bevy::prelude::*;
use bevy_rand::{prelude::WyRand, resource::GlobalEntropy};
use enum_iterator::{all, Sequence};
use rand::{seq::SliceRandom, Rng};

use crate::{
    enemy::Enemy, health::DeathEvent, loading::TextureAssets, player::Player, weapon::Coord2D,
    GameState,
};

pub struct PowerupPlugin;

impl Plugin for PowerupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_pickup, player_pickup).run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component)]
struct Pickup;

fn spawn_pickup(
    mut commands: Commands,
    assets: Res<TextureAssets>,
    mut death_events: EventReader<DeathEvent>,
    enemies: Query<&Transform, With<Enemy>>,
) {
    for death in death_events.read() {
        // look for where the enemies body is
        let entity = death.0;
        if let Ok(position) = enemies.get(entity) {
            commands.spawn((
                SpriteBundle {
                    texture: assets.bevy.clone(),
                    transform: *position,
                    ..Default::default()
                },
                Pickup,
            ));
        }
    }
}

#[derive(Component)]
struct ChooserMenu;

#[derive(Component)]
struct Choice(PowerUpType);

fn player_pickup(
    mut commands: Commands,
    mut players: Query<(&mut Powerups, &Transform), With<Player>>,
    pickups: Query<(Entity, &Transform), With<Pickup>>,
    mut rand: ResMut<GlobalEntropy<WyRand>>,
) {
    for (powerup, player) in players.iter_mut() {
        for (pickup, pickup_location) in pickups.iter() {
            // check if they overlap
            if (*Coord2D::from(player.translation) - *Coord2D::from(pickup_location.translation))
                .length_squared()
                < 100.0
            {
                commands.entity(pickup).despawn();
                // if the user has no slots left, we should show just the items that they have
                // otherwise, we should backfill with other types of items to choose from
                let mut choices = powerup.current_powerups().collect::<Vec<_>>();
                let unused = powerup.get_unused_powerup_types();
                let unused_slots = powerup.unused_slots();
                for _num in 0..unused_slots {
                    let mut unused = unused.clone();
                    choices.append(&mut unused);
                }
                let rand = &mut *rand;
                let choices = choices.choose_multiple(rand, 3);
                commands
                    .spawn((
                        NodeBundle {
                            style: Style {
                                align_items: AlignItems::Center,
                                justify_items: JustifyItems::Center,
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ChooserMenu,
                    ))
                    .with_children(|c| {
                        for choice in choices {
                            c.spawn((
                                ButtonBundle {
                                    background_color: BackgroundColor(Color::rgb(1.0, 0.0, 0.0)),
                                    ..Default::default()
                                },
                                Choice(*choice),
                            ))
                            .with_children(|c| {
                                c.spawn(TextBundle::from_section(
                                    format!("{:?}", choice),
                                    TextStyle {
                                        font_size: 20.0,
                                        ..Default::default()
                                    },
                                ));
                            });
                        }
                    });
            }
        }
    }
}

fn choices(mut commands: Commands, buttons: Query<(Interaction, Choice), Changed<Interaction>>) {}

#[derive(Component)]
struct Powerups([Option<Powerup>; 6]);

impl Powerups {
    fn unused_slots(&self) -> usize {
        self.0.iter().filter(|f| f.is_none()).count()
    }

    fn get_powerup_mut(&mut self, power_type: PowerUpType) -> Option<&mut Powerup> {
        self.0
            .iter_mut()
            .flatten()
            .find(|power| power.power == power_type)
    }

    fn get_powerup(&self, power_type: PowerUpType) -> Option<&Powerup> {
        self.0
            .iter()
            .flatten()
            .find(|power| power.power == power_type)
    }

    fn current_powerups(&self) -> impl Iterator<Item = PowerUpType> + '_ {
        self.0.iter().flatten().map(|t| t.power)
    }

    fn get_unused_powerup_types(&self) -> Vec<PowerUpType> {
        let mut types: Vec<PowerUpType> = all::<PowerUpType>().collect();
        types.retain(|t| self.get_powerup(*t).is_none());
        types
    }
}

struct Powerup {
    power: PowerUpType,
    level: u8,
}

/// [`PowerUpType`] is just an enumeration of each type of powerup that a ship can have- players and enemies share these power ups
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Sequence)]
enum PowerUpType {
    /// Shoots a single blob
    PeaShooter,
    /// Shoots a constant barrage of fire
    MachineGun,
    /// Passes through multiple enemies and curves back to you
    Boomerrang,
    /// Follows the heat signature of an enemy and explodes
    HeatSeeker,
    /// Drops a giant nuke directly ontop of you
    TacticalNuke,
    /// Fires a beam directly in front of you
    LazerCannon,
    /// Sprays small leaves that enemies slip on
    LeafBlower,
    /// Stationary floating mines that enemies run over
    AirMines,
    /// Companion drones that fire pea shooters separately
    Drones,
    /// Heals self gradually
    Nanobots,
    /// prevents the enemy from targeting you while active
    Flares,
    /// Adds extra health ontop of your current shield
    Overshield,
    /// Adds total health
    Armor,
    /// Duplicates yourself and copies all weapons
    Squadron,
    /// Increases the number of projectiles
    ExtraProjectile,
    /// Increases targeting distance
    SatelliteSupport,
    /// Increases movement speed
    EnergySoda,
}
