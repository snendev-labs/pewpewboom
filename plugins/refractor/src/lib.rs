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
    lasers::{Consumption, Direction, Position, Refraction},
    Owner, Tile, TileParameters, TilePlugin,
};

pub struct RefractorPlugin;

impl Plugin for RefractorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilePlugin::<RefractorTile>::default())
            .add_systems(Update, Self::update_marker);
        app.define_merchandise::<RefractorTile>();
    }
}

impl RefractorPlugin {
    fn update_marker(
        tiles: Query<&Direction, (With<RefractorTile>, Changed<Direction>)>,
        mut markers: Query<(&Parent, &mut Transform), With<RefractorMarker>>,
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
                };
                info!("Changed refractor marker transform");
            }
        }
    }
}

#[derive(Debug)]
#[derive(Component, Reflect)]
pub struct RefractorTile;

impl Tile for RefractorTile {
    fn spawn(position: Position, player: Entity, _game: Entity) -> impl Command {
        RefractorSpawn { position, player }
    }

    fn material(_asset_server: &AssetServer) -> ColorMaterial {
        Color::Srgba(palettes::css::CORNFLOWER_BLUE).into()
    }

    fn activate(
        &self,
        entity: Entity,
        parameters: TileParameters,
        _shooter: Option<Entity>,
    ) -> impl Command {
        RefractorActivate {
            tile: entity,
            position: parameters.position,
            direction: parameters
                .direction
                .unwrap_or_else(|| panic!("Refractor needs a direction")),
        }
    }

    fn on_hit(&self, entity: Entity, strength: usize, _shooter: Entity) -> Option<impl Command> {
        Some(RefractorOnHit {
            tile: entity,
            strength,
        })
    }
}

impl Merchandise for RefractorTile {
    const PRICE: Money = Money::new(5);
    const NAME: &'static str = "Refractor Tower";

    fn material(asset_server: &AssetServer) -> ColorMaterial {
        let mut base = <Self as Tile>::material(asset_server);
        base.color.set_alpha(0.6);
        base
    }
}

pub struct RefractorSpawn {
    position: Position,
    player: Entity,
}

impl Command for RefractorSpawn {
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

        let rectangle = meshes.add(Rectangle::new(60., 5.));
        let black = materials.add(Color::BLACK);
        let triangle = meshes.add(Triangle2d::new(
            Vec2::new(-5., -8.),
            Vec2::new(5., -8.),
            Vec2::new(0., -18.),
        ));
        let red = materials.add(Color::Srgba(bevy::color::palettes::css::RED));

        if let Some(game) = world.get::<InGame>(self.player) {
            world
                .spawn((
                    RefractorTile,
                    self.position,
                    Direction::default(),
                    Owner::new(self.player),
                    game.clone(),
                    Transform::from_translation(translation),
                    GlobalTransform::from_translation(translation),
                    JustPurchased,
                ))
                .with_children(|builder| {
                    info!("Spawning child marker for refractor");
                    builder.spawn((
                        RefractorMarker,
                        MaterialMesh2dBundle {
                            mesh: rectangle.into(),
                            material: black,
                            transform: Transform::default(),
                            ..default()
                        },
                    ));

                    builder.spawn((
                        RefractorMarker,
                        MaterialMesh2dBundle {
                            mesh: triangle.into(),
                            material: red,
                            transform: Transform::default(),
                            ..default()
                        },
                    ));
                });
        }
    }
}

pub struct RefractorActivate {
    tile: Entity,
    position: Position,
    direction: Direction,
}

impl Command for RefractorActivate {
    fn apply(self, world: &mut World) {
        world.spawn((
            Refraction::new(self.direction),
            Consumption::bundle(
                self.tile,
                self.direction.front_directions().to_vec(),
                self.position.clone(),
            ),
        ));
    }
}

pub struct RefractorOnHit {
    tile: Entity,
    strength: usize,
}

impl Command for RefractorOnHit {
    fn apply(self, world: &mut World) {
        let mut consumer_health = world
            .get_mut::<Health>(self.tile)
            .expect("RefractorOnHit entity should have Health component added to it");
        **consumer_health -= self.strength;
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
pub struct RefractorMarker;
