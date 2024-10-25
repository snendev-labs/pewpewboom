use std::f32::consts::PI;

use bevy::{
    color::palettes,
    ecs::{system::SystemState, world::Command},
    prelude::*,
    sprite::MaterialMesh2dBundle,
};

use game_loop::InGame;
use merchandise::{MerchAppExt, Merchandise, Money};
use tilemap::TilemapLayout;
use tiles::{
    lasers::{Position, Rotation},
    Owner, Tile, TileParameters, TilePlugin,
};

pub struct RotaterPlugin;

impl Plugin for RotaterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilePlugin::<RotaterTile>::default());
        app.define_merchandise::<RotaterTile>();
    }
}

#[derive(Debug)]
#[derive(Component, Reflect)]
pub struct RotaterTile;

impl Tile for RotaterTile {
    fn spawn(parameters: TileParameters, player: Entity) -> impl Command {
        RotaterSpawn {
            position: parameters.position,
            rotation: parameters.rotation.unwrap_or_default(),
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
        RotaterActivate {
            position: parameters.position,
            rotation: parameters
                .rotation
                .unwrap_or_else(|| panic!("Rotator needs a rotation")),
        }
    }
}

impl Merchandise for RotaterTile {
    const PRICE: Money = Money::new(5);
    const NAME: &'static str = "Rotater";

    fn material(asset_server: &AssetServer) -> ColorMaterial {
        let mut base = <Self as Tile>::material(asset_server);
        base.color.set_alpha(0.6);
        base
    }
}

pub struct RotaterSpawn {
    position: Position,
    rotation: Rotation,
    player: Entity,
}

impl Command for RotaterSpawn {
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

        let sector = meshes.add(CircularSector::new(
            40.,
            ((self.rotation.get() % 6) as f32) * (PI / 6.),
        ));
        let black = materials.add(Color::BLACK);

        if let Some(game) = world.get::<InGame>(self.player) {
            world
                .spawn((
                    RotaterTile,
                    self.position,
                    self.rotation,
                    Owner::new(self.player),
                    game.clone(),
                    Transform::default(),
                    GlobalTransform::default(),
                ))
                .with_children(|builder| {
                    builder.spawn(MaterialMesh2dBundle {
                        mesh: sector.into(),
                        material: black,
                        transform: Transform::from_translation(translation),
                        ..default()
                    });
                });
        }
    }
}

pub struct RotaterActivate {
    position: Position,
    rotation: Rotation,
}

impl Command for RotaterActivate {
    fn apply(self, world: &mut World) {
        world.spawn((Rotation::new(self.rotation.get()), self.position));
    }
}
