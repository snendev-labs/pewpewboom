use thiserror::Error;

use bevy::{prelude::*, reflect::GetTypeRegistration};

use bevy_anyhow_alert::*;

mod components;
pub use components::*;
mod registry;
pub use registry::*;

pub struct MerchPlugin;

impl Plugin for MerchPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MerchRegistry>();
        app.add_systems(
            Update,
            Self::try_purchase.anyhow_alerts().in_set(MerchSystems),
        );
    }
}

impl MerchPlugin {
    fn try_purchase(
        mut commands: Commands,
        mut shoppers: Query<(Entity, &mut Money, Option<&Purchase>), With<Shopper>>,
    ) -> ResultVec<(), PurchaseError> {
        let purchases = shoppers
            .iter_mut()
            .map(|(shopper, money, purchase)| {
                purchase
                    .ok_or(PurchaseError::NoSelection(shopper))
                    .and_then(|purchase| {
                        if *money > purchase.price() {
                            Ok((shopper, money, purchase))
                        } else {
                            Err(PurchaseError::NotEnoughMoney {
                                shopper,
                                cost: purchase.price(),
                                money: *money,
                            })
                        }
                    })
            })
            .collect::<Vec<_>>();

        if purchases.iter().all(Result::is_ok) {
            for (entity, mut money, purchase) in purchases.into_iter().filter_map(Result::ok) {
                **money = money.saturating_sub(*purchase.price());
                commands.entity(entity).remove::<Purchase>();
            }
            Ok(())
        } else {
            Err(purchases.into_iter().filter_map(Result::err).collect())
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub struct MerchSystems;

#[derive(Debug)]
#[derive(Error)]
pub enum PurchaseError {
    #[error("No selection made for {0}")]
    NoSelection(Entity),
    #[error("N")]
    NotEnoughMoney {
        shopper: Entity,
        cost: Money,
        money: Money,
    },
}

pub trait Merchandise {
    const PRICE: Money;
}

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
        registry.register::<T>().unwrap();
    }
}
