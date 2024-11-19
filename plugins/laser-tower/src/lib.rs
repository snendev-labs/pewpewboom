use std::f32::consts::PI;

use bevy::{
    color::palettes,
    ecs::{system::SystemState, world::Command},
    prelude::*,
    sprite::MaterialMesh2dBundle,
};

use game_loop::InGame;
use health::Health;
use merchandise::{MerchAppExt, Merchandise, Money};
use popups::PopupEvent;
use shop::JustPurchased;
use tilemap::TilemapLayout;
use tiles::{
    lasers::{Consumption, Direction, Laser, Position, Shooter},
    Owner, Tile, TileParameters, TilePlugin,
};

pub struct LaserTowerPlugin;

impl Plugin for LaserTowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilePlugin::<LaserTower>::default())
            .add_systems(Update, Self::update_marker);
        app.define_merchandise::<LaserTower>();
    }
}

impl LaserTowerPlugin {
    fn update_marker(
        tiles: Query<&Direction, (With<LaserTower>, Changed<Direction>)>,
        mut markers: Query<(&Parent, &mut Transform), With<LaserTowerMarker>>,
    ) {
        for (parent, mut transform) in &mut markers {
            if let Ok(direction) = tiles.get(**parent) {
                *transform = match *direction {
                    Direction::North => Transform::IDENTITY,
                    Direction::Northwest => {
                        Transform::from_rotation(Quat::from_rotation_z(PI / 3.))
                    }
                    Direction::Southwest => {
                        Transform::from_rotation(Quat::from_rotation_z(2. * PI / 3.))
                    }
                    Direction::South => Transform::from_rotation(Quat::from_rotation_z(PI)),
                    Direction::Southeast => {
                        Transform::from_rotation(Quat::from_rotation_z(4. * PI / 3.))
                    }
                    Direction::Northeast => {
                        Transform::from_rotation(Quat::from_rotation_z(5. * PI / 3.))
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
#[derive(Component, Reflect)]
pub struct LaserTower;

impl Tile for LaserTower {
    fn spawn(position: Position, player: Entity, _game: Entity) -> impl Command {
        LaserTowerSpawn { position, player }
    }

    fn material(_asset_server: &AssetServer) -> ColorMaterial {
        Color::Srgba(palettes::css::CRIMSON).into()
    }

    fn activate(
        &self,
        tile: Entity,
        parameters: TileParameters,
        shooter: Option<Entity>,
    ) -> impl Command {
        LaserTowerActivate {
            tile,
            position: parameters.position,
            direction: parameters
                .direction
                .unwrap_or_else(|| panic!("Laser tower needs a direction")),
            shooter: shooter
                .unwrap_or_else(|| panic!("Laser tower needs to have a owner to shoot")),
        }
    }
}

impl Merchandise for LaserTower {
    const PRICE: Money = Money::new(10);
    const NAME: &'static str = "Laser Tower";

    fn material(asset_server: &AssetServer) -> ColorMaterial {
        let mut base = <Self as Tile>::material(asset_server);
        base.color.set_alpha(0.6);
        base
    }
}

pub struct LaserTowerSpawn {
    position: Position,
    player: Entity,
}

impl Command for LaserTowerSpawn {
    fn apply(self, world: &mut World) {
        let mut system_state: SystemState<(
            Query<&TilemapLayout>,
            ResMut<Assets<Mesh>>,
            ResMut<Assets<ColorMaterial>>,
        )> = SystemState::new(world);

        let (layout, mut meshes, mut materials) = system_state.get_mut(world);

        let Ok(translation) = layout
            .get_single()
            .and_then(|layout| Ok(layout.hex_to_world_pos(*self.position).extend(11.)))
        else {
            info!("Did not get the single tilemap layout for the game");
            return;
        };

        let triangle = meshes.add(Triangle2d::new(
            Vec2::new(-5., 0.),
            Vec2::new(5., 0.),
            Vec2::new(0., 40.),
        ));
        let white = materials.add(Color::WHITE);

        if let Some(game) = world.get::<InGame>(self.player) {
            world
                .spawn((
                    LaserTower,
                    self.position,
                    Direction::default(),
                    Owner::new(self.player),
                    game.clone(),
                    // Need to add dummy Transform and GlobalTransform to parent otherwise the child marker will not render due to bevy issue
                    // Copied solution in all other markers like this one
                    Transform::from_translation(translation),
                    GlobalTransform::from_translation(translation),
                    JustPurchased,
                ))
                .with_children(|builder| {
                    builder.spawn((
                        LaserTowerMarker,
                        MaterialMesh2dBundle {
                            mesh: triangle.into(),
                            material: white,
                            transform: Transform::default(),
                            ..default()
                        },
                    ));
                });
        }
    }
}

#[derive(Clone, Debug)]
pub struct LaserTowerActivate {
    tile: Entity,
    position: Position,
    direction: Direction,
    shooter: Entity,
}

impl Command for LaserTowerActivate {
    fn apply(self, world: &mut World) {
        world.spawn(Consumption::bundle(
            self.tile,
            Direction::ALL.to_vec(),
            self.position,
        ));
        world.spawn((
            Laser,
            self.position,
            self.direction,
            Shooter::new(self.shooter),
        ));
    }
}

#[derive(Clone, Debug)]
pub struct LaserTowerOnHit {
    tile: Entity,
    strength: usize,
}

impl Command for LaserTowerOnHit {
    fn apply(self, world: &mut World) {
        let mut laser_tower_health = world
            .get_mut::<Health>(self.tile)
            .expect("Laser tower to have Health");
        **laser_tower_health -= self.strength;
        world.trigger_targets(
            PopupEvent {
                text: String::from(format!("-{}", self.strength)),
            },
            self.tile,
        );
    }
}

#[derive(Clone, Copy, Debug)]
#[derive(Component)]
pub struct LaserTowerMarker;
