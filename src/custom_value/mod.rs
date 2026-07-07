mod registry;

use std::sync::RwLock;

use hashbrown::HashMap;
pub use registry::HandlebarsRegistry;
use uuid::Uuid;

pub trait CustomReference: Clone {
    type Data;
    const NAME: &str;

    fn uuid(&self) -> &Uuid;
}

#[derive(Debug)]
pub struct CustomEntry<C: CustomReference> {
    pub reference: C,
    pub refcount: usize,
    pub data: C::Data,
}

#[derive(Default, Debug)]
pub struct CustomCollections {
    pub registries: RwLock<HashMap<Uuid, CustomEntry<HandlebarsRegistry>>>,
}

impl CustomCollections {
    pub fn is_empty(&self) -> bool {
        self.registries.read().unwrap().is_empty()
    }
}
