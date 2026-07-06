mod registry;

use hashbrown::HashMap;
pub use registry::HandlebarsRegistry;
use uuid::Uuid;

pub trait CustomReference: Clone {
    type Data;

    fn uuid(&self) -> Uuid;
}

#[derive(Debug)]
pub struct CustomEntry<C: CustomReference> {
    pub reference: C,
    pub refcount: usize,
    pub data: C::Data,
}

#[derive(Default, Debug)]
pub struct CustomCollections {
    pub registries: HashMap<Uuid, CustomEntry<HandlebarsRegistry>>,
}
