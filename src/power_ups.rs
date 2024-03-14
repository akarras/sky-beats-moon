use bevy::prelude::*;
use bevy_rand::{prelude::WyRand, resource::GlobalEntropy};
use enum_iterator::{all, Sequence};
use rand::{seq::SliceRandom, Rng};

use crate::{
    enemy::Enemy,
    health::DeathEvent,
    loading::TextureAssets,
    overshield::Overshield,
    player::Player,
    weapon::{Bile, Coord2D, MachineGun, PeaShooter, Sniper, SpecialMunitions},
    GameState,
};

pub struct PowerupPlugin;

impl Plugin for PowerupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_pickup, player_pickup, powerup_manager).run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnEnter(GameState::Menu), cleanup_pickups)
        .add_systems(OnEnter(GameState::Chooser), add_choice_menu)
        .add_systems(OnExit(GameState::Chooser), remove_choice_menu)
        .add_systems(Update, choices.run_if(in_state(GameState::Chooser)));
    }
}

#[derive(Component)]
struct Pickup;

fn spawn_pickup(
    mut commands: Commands,
    assets: Res<TextureAssets>,
    mut death_events: EventReader<DeathEvent>,
    enemies: Query<&Transform, With<Enemy>>,
    mut rand: ResMut<GlobalEntropy<WyRand>>,
) {
    for death in death_events.read() {
        // look for where the enemies body is
        let entity = death.0;
        if let Ok(position) = enemies.get(entity) {
            if rand.gen_bool(0.3) {
                commands.spawn((
                    SpriteBundle {
                        texture: assets.supply_crate.clone(),
                        transform: *position,
                        ..Default::default()
                    },
                    Pickup,
                ));
            }
        }
    }
}

fn cleanup_pickups(mut commands: Commands, pickups: Query<Entity, With<Pickup>>) {
    for pickup in pickups.iter() {
        commands.entity(pickup).despawn();
    }
}

#[derive(Component)]
struct ChoiceMenu;

#[derive(Component)]
struct Choice(PowerUpType);

fn player_pickup(
    mut commands: Commands,
    players: Query<&Transform, With<Player>>,
    pickups: Query<(Entity, &Transform), With<Pickup>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for player in players.iter() {
        for (pickup, pickup_location) in pickups.iter() {
            // check if they overlap
            if (*Coord2D::from(player.translation) - *Coord2D::from(pickup_location.translation))
                .length_squared()
                < 1000.0
            {
                commands.entity(pickup).despawn();
                next_state.set(GameState::Chooser);
            }
        }
    }
}

fn choices(
    buttons: Query<(&Interaction, &Choice), Changed<Interaction>>,
    mut player: Query<&mut Powerups, With<Player>>,
    mut state: ResMut<NextState<GameState>>,
) {
    for (button, choice) in buttons.iter() {
        if let Interaction::Pressed = button {
            let mut player = player.single_mut();
            player.add_powerup(choice.0);
            state.set(GameState::Playing);
        }
    }
}

#[derive(Component, Default)]
pub struct Powerups(pub [Option<Powerup>; 6]);

fn powerup_manager(
    mut commands: Commands,
    powerups: Query<(Entity, &Powerups), Changed<Powerups>>,
) {
    for (entity, powerups) in &powerups {
        for powerup in powerups.0.iter().flatten() {
            let level = powerup.level;
            let mut entity = commands.entity(entity);
            match powerup.power {
                PowerUpType::MachineGun => entity.insert(MachineGun::new(level)),
                PowerUpType::PeaShooter => entity.insert(PeaShooter::new(level)),
                PowerUpType::Sniper => entity.insert(Sniper::new(level)),
                PowerUpType::SpecialMunitions => entity.insert(SpecialMunitions::new(level)),
                PowerUpType::Overshield => entity.insert(Overshield::new(level)),
                PowerUpType::Bile => entity.insert(Bile::new(level)),
            };
        }
    }
}

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

    fn add_powerup(&mut self, power_type: PowerUpType) {
        if let Some(power) = self.get_powerup_mut(power_type) {
            power.level += 1;
        } else {
            for item in self.0.iter_mut() {
                if item.is_none() {
                    *item = Some(Powerup {
                        power: power_type,
                        level: 1,
                    });
                    break;
                }
            }
        }
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

fn add_choice_menu(
    mut commands: Commands,
    mut rand: ResMut<GlobalEntropy<WyRand>>,
    mut player: Query<&mut Powerups, With<Player>>,
) {
    // if the user has no slots left, we should show just the items that they have
    // otherwise, we should backfill with other types of items to choose from
    let powerup = player.single_mut();
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
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..Default::default()
                },
                background_color: BackgroundColor(Color::rgb(0.8, 0.4, 0.0)),
                ..Default::default()
            },
            ChoiceMenu,
        ))
        .with_children(|c| {
            c.spawn(TextBundle::from_section(
                "Choose a powerup!",
                TextStyle {
                    font_size: 42.0,
                    ..Default::default()
                },
            ));
            for choice in choices {
                c.spawn((
                    ButtonBundle {
                        background_color: BackgroundColor(Color::rgb(0.0, 0.3, 0.0)),
                        style: Style {
                            width: Val::Vw(60.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    Choice(*choice),
                ))
                .with_children(|c| {
                    c.spawn(TextBundle::from_section(
                        format!("{:?}", choice),
                        TextStyle {
                            font_size: 60.0,
                            ..Default::default()
                        },
                    ));
                });
            }
        });
}

fn remove_choice_menu(mut commands: Commands, menu: Query<Entity, With<ChoiceMenu>>) {
    for menu in menu.iter() {
        commands.entity(menu).despawn_recursive();
    }
}

pub struct Powerup {
    pub power: PowerUpType,
    pub level: u8,
}

/// [`PowerUpType`] is just an enumeration of each type of powerup that a ship can have- players and enemies share these power ups
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Sequence)]
pub enum PowerUpType {
    // /// Shoots a single blob
    PeaShooter,
    /// Shoots a constant barrage of fire
    MachineGun,
    /// Shoots a single high power bullet
    Sniper,
    /// Shoots a spray of green bile
    Bile,
    // /// Passes through multiple enemies and curves back to you
    // Boomerrang,
    // /// Follows the heat signature of an enemy and explodes
    // HeatSeeker,
    // /// Drops a giant nuke directly ontop of you
    // TacticalNuke,
    // /// Fires a beam directly in front of you
    // LazerCannon,
    // /// Sprays small leaves that enemies slip on
    // LeafBlower,
    // /// Stationary floating mines that enemies run over
    // AirMines,
    // /// Companion drones that fire pea shooters separately
    // Drones,
    // /// Heals self gradually
    // Nanobots,
    // /// prevents the enemy from targeting you while active
    // Flares,
    /// Adds extra health ontop of your current health
    Overshield,
    /// Increases the damage of all weapons
    SpecialMunitions,
    // Increases the number of enemies that projectiles will pass through
    // /// Adds total health
    // Armor,
    // /// Duplicates yourself and copies all weapons
    // Squadron,
    // /// Increases the number of projectiles
    // ExtraProjectile,
    // /// Increases targeting distance
    // SatelliteSupport,
    // /// Increases movement speed
    // EnergySoda,
}
