use bevy::{
    color::palettes,
    ecs::{system::SystemState, world::Command},
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use game_loop::InGame;
use health::Health;
use merchandise::{MerchAppExt, Merchandise, Money};
use tilemap::TilemapLayout;
use tiles::{
    lasers::{Consumption, Direction, Laser, Position, Shooter},
    Owner, Tile, TileParameters, TilePlugin,
};

pub struct LaserTowerPlugin;

impl Plugin for LaserTowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilePlugin::<LaserTower>::default());
        app.define_merchandise::<LaserTower>();
    }
}

#[derive(Debug)]
#[derive(Component, Reflect)]
pub struct LaserTower;

impl Tile for LaserTower {
    fn spawn(parameters: TileParameters, player: Entity) -> impl Command {
        LaserTowerSpawn {
            position: parameters.position,
            direction: parameters.direction.unwrap_or_default(),
            player,
        }
    }

    fn material(_asset_server: &AssetServer) -> ColorMaterial {
        ColorMaterial::from_color(Color::Srgba(palettes::css::CRIMSON))
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
    direction: Direction,
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
            .and_then(|layout| Ok(layout.hex_to_world_pos(*self.position).extend(1.)))
        else {
            info!("Did not get the single tilemap layout for the game");
            return;
        };

        let mesh = Mesh2dHandle(meshes.add(Rectangle::new(40., 5.)));
        let material = materials.add(Color::BLACK);

        if let Some(game) = world.get::<InGame>(self.player) {
            world
                .spawn((
                    LaserTower,
                    self.position,
                    self.direction,
                    Owner::new(self.player),
                    game.clone(),
                ))
                .with_children(|_| {
                    MaterialMesh2dBundle {
                        mesh,
                        material,
                        transform: Transform::from_translation(translation),
                        ..default()
                    };
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
    }
}
