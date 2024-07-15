use std::any::TypeId;

use bevy::{
    prelude::*,
    reflect::GetTypeRegistration,
    utils::{HashMap, TypeIdMap},
};

use crate::{Merch, MerchId, Merchandise};

#[derive(Debug, Default)]
#[derive(Resource)]
pub struct MerchRegistry {
    by_type: TypeIdMap<Merch>,
    by_id: HashMap<MerchId, (TypeId, Merch)>,
    next_id: MerchId,
}

impl MerchRegistry {
    pub fn register<T>(&mut self) -> Result<Merch, Merch>
    where
        T: Merchandise + GetTypeRegistration,
    {
        let merch_id = self.next_id;
        let type_id = T::get_type_registration().type_id();
        let merch = Merch::new(merch_id, <T as Merchandise>::PRICE);
        let replaced_merch = self.by_type.insert(type_id, merch.clone());
        let replaced_type = self.by_id.insert(merch_id, (type_id, merch));
        *self.next_id += 1;
        if let Some(merch) = replaced_merch.or(replaced_type.map(|(_, merch)| merch)) {
            Err(merch)
        } else {
            Ok(merch)
        }
    }

    pub fn get<T: GetTypeRegistration>(&self) -> Option<&Merch> {
        self.by_type.get(&T::get_type_registration().type_id())
    }

    pub fn get_type(&self, merch: &MerchId) -> Option<&TypeId> {
        self.by_id.get(merch).map(|(id, _)| id)
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a TypeId, &'a Merch)> + 'a {
        self.by_type.iter()
    }
}
