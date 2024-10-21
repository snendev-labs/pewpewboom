use thiserror::Error;

use bevy::{prelude::*, reflect::GetTypeRegistration, utils::HashMap};

use bevy_anyhow_alert::*;

use game_loop::Player;
use tiles::TileSpawnEvent;

mod components;
pub use components::*;
mod registry;
pub use registry::*;

pub trait Merchandise {
    const PRICE: Money;
    const NAME: &'static str;

    fn material(asset_server: &AssetServer) -> ColorMaterial;
}

pub struct MerchPlugin;

impl Plugin for MerchPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Purchase>();
        app.init_resource::<MerchRegistry>();
        app.init_resource::<MerchMaterials>();
        app.add_systems(
            Update,
            (Self::spawn_shoppers, Self::handle_purchases.anyhow_alerts()).in_set(MerchSystems),
        );
    }
}

impl MerchPlugin {
    fn spawn_shoppers(mut commands: Commands, added_players: Query<Entity, Added<Player>>) {
        for player in &added_players {
            commands.entity(player).insert((Shopper, Money::new(50)));
        }
    }

    fn handle_purchases(
        mut purchases: EventReader<Purchase>,
        mut tile_spawns: EventWriter<TileSpawnEvent>,
        registry: Res<MerchRegistry>,
        mut shoppers: Query<&mut Money, With<Shopper>>,
    ) -> ResultVec<(), PurchaseError> {
        let mut errors = vec![];
        for Purchase {
            buyer,
            merch,
            on_tile,
        } in purchases.read()
        {
            info!("Handling purchase");
            let Ok(mut money) = shoppers.get_mut(*buyer) else {
                continue;
            };
            info!("Player money recognized");
            let cost = merch.price();
            if **money >= *cost {
                if let Some(tile_id) = registry.get_type(&merch.id()) {
                    **money = money.saturating_sub(*cost);
                    info!(
                        "Tile spawn event on tile {:?} for tile type {:?}",
                        *on_tile, *tile_id
                    );
                    tile_spawns.send(TileSpawnEvent {
                        tile_id: *tile_id,
                        on_tile: *on_tile,
                        player: *buyer,
                    });
                } else {
                    info!("Unknown merch error");
                    errors.push(PurchaseError::UnknownMerch {
                        merch_id: merch.id(),
                    })
                }
            } else {
                info!("Insufficient funds purchase error");
                errors.push(PurchaseError::NotEnoughMoney {
                    shopper: *buyer,
                    cost,
                    money: *money,
                })
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub struct MerchSystems;

#[derive(Clone, Debug)]
#[derive(Event)]
#[derive(Reflect)]
pub struct Purchase {
    buyer: Entity,
    merch: Merch,
    on_tile: Entity,
}

impl Purchase {
    pub fn new(buyer: Entity, merch: Merch, on_tile: Entity) -> Self {
        Purchase {
            buyer,
            merch,
            on_tile,
        }
    }
}

#[derive(Debug)]
#[derive(Error)]
#[derive(Reflect)]
pub enum PurchaseError {
    #[error("N")]
    NotEnoughMoney {
        shopper: Entity,
        cost: Money,
        money: Money,
    },
    #[error("N")]
    UnknownMerch { merch_id: MerchId },
}

#[derive(Debug, Default)]
#[derive(Deref, DerefMut, Resource, Reflect)]
pub struct MerchMaterials(HashMap<MerchId, Handle<ColorMaterial>>);

pub trait MerchAppExt {
    fn define_merchandise<T>(&mut self)
    where
        T: Component + GetTypeRegistration + Merchandise,
    {
    }
}

impl MerchAppExt for App {
    fn define_merchandise<T>(&mut self)
    where
        T: Component + GetTypeRegistration + Merchandise,
    {
        let mut registry = self.world_mut().resource_mut::<MerchRegistry>();
        let merch = match registry.register::<T>() {
            Ok(merch) => merch,
            Err(merch) => merch,
        };
        let asset_server = self.world().resource::<AssetServer>();
        let material = T::material(asset_server);
        let mut materials = self.world_mut().resource_mut::<Assets<ColorMaterial>>();
        let handle = materials.add(material);
        let mut textures = self.world_mut().resource_mut::<MerchMaterials>();
        textures.insert(merch.id(), handle);
    }
}
