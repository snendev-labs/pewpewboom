use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component, Reflect)]
pub struct Shopper;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Component, Deref, DerefMut, Reflect)]
pub struct Money(usize);

impl std::fmt::Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}", **self)
    }
}

impl Money {
    pub const fn new(value: usize) -> Self {
        Money(value)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[derive(Component, Deref, DerefMut, Reflect)]
pub struct MerchId(usize);

#[derive(Clone, Debug, Default)]
#[derive(Component, Reflect)]
pub struct Merch {
    id: MerchId,
    price: Money,
}

impl Merch {
    pub const fn new(id: MerchId, price: Money) -> Self {
        Merch { id, price }
    }

    pub fn id(&self) -> MerchId {
        self.id
    }

    pub fn price(&self) -> Money {
        self.price
    }
}
