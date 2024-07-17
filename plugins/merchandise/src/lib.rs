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
        app.add_event::<Purchase>();
        app.init_resource::<MerchRegistry>();
        app.add_systems(
            Update,
            Self::handle_purchases.anyhow_alerts().in_set(MerchSystems),
        );
    }
}

impl MerchPlugin {
    fn handle_purchases(
        mut purchases: EventReader<Purchase>,
        mut shoppers: Query<&mut Money, With<Shopper>>,
    ) -> ResultVec<(), PurchaseError> {
        let mut errors = vec![];
        for Purchase { buyer, merch } in purchases.read() {
            let Ok(mut money) = shoppers.get_mut(*buyer) else {
                continue;
            };
            let cost = merch.price();
            if **money >= *cost {
                **money = money.saturating_sub(*cost);
            } else {
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
}

impl Purchase {
    pub fn new(buyer: Entity, merch: Merch) -> Self {
        Purchase { buyer, merch }
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
