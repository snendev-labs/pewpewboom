use bevy::{
    color::palettes,
    ecs::{system::SystemState, world::Command},
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use game_loop::InGame;
use merchandise::{MerchAppExt, Merchandise, Money};
use tilemap::TilemapLayout;
use tiles::{
    lasers::{Direction, Position, Reflection},
    Owner, Tile, TileParameters, TilePlugin,
};

pub struct ReflectorPlugin;

impl Plugin for ReflectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilePlugin::<ReflectorTile>::default());
        app.define_merchandise::<ReflectorTile>();
    }
}

#[derive(Debug)]
#[derive(Component, Reflect)]
pub struct ReflectorTile;

impl Tile for ReflectorTile {
    fn spawn(parameters: TileParameters, player: Entity) -> impl Command {
        ReflectorSpawn {
            position: parameters.position,
            direction: parameters.direction.unwrap_or_default(),
            player,
        }
    }

    fn material(_asset_server: &AssetServer) -> ColorMaterial {
        ColorMaterial::from_color(Color::Srgba(palettes::css::CADET_BLUE))
    }

    fn activate(
        &self,
        _entity: Entity,
        parameters: TileParameters,
        _shooter: Option<Entity>,
    ) -> impl Command {
        ReflectorActivate {
            position: parameters.position,
            direction: parameters
                .direction
                .unwrap_or_else(|| panic!("Reflector needs a direction")),
        }
    }
}

impl Merchandise for ReflectorTile {
    const PRICE: Money = Money::new(5);
    const NAME: &'static str = "Reflector Tower";

    fn material(asset_server: &AssetServer) -> ColorMaterial {
        let mut base = <Self as Tile>::material(asset_server);
        base.color.set_alpha(0.6);
        base
    }
}

pub struct ReflectorSpawn {
    position: Position,
    direction: Direction,
    player: Entity,
}

impl Command for ReflectorSpawn {
    fn apply(self, world: &mut World) {
        let mut system_state: SystemState<(
            Query<&TilemapLayout>,
            ResMut<Assets<Mesh>>,
            ResMut<Assets<ColorMaterial>>,
        )> = SystemState::new(world);

        let (layout, mut meshes, mut materials) = system_state.get_mut(world);

        let Ok(translation) = layout
            .get_single()
            .and_then(|layout| Ok(layout.hex_to_world_pos(*self.position).extend(0.)))
        else {
            info!("Did not get the single tilemap layout for the game");
            return;
        };

        let mesh = Mesh2dHandle(meshes.add(Rectangle::new(50., 5.)));
        let material = materials.add(Color::BLACK);

        if let Some(game) = world.get::<InGame>(self.player) {
            world
                .spawn((
                    ReflectorTile,
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

pub struct ReflectorActivate {
    position: Position,
    direction: Direction,
}

impl Command for ReflectorActivate {
    fn apply(self, world: &mut World) {
        world.spawn((Reflection::new(self.direction), self.position));
    }
}
