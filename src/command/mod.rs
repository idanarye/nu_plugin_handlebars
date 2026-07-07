use std::any::TypeId;
use std::sync::RwLock;

use hashbrown::HashMap;
use hashbrown::hash_map::Entry;
use nu_plugin::{EngineInterface, Plugin, PluginCommand};
use nu_protocol::{CustomValue, LabeledError, PipelineData, Signature, Value};

use crate::custom_value::{CustomCollections, CustomReference, HandlebarsRegistry};

mod eval;
mod helper;
mod list;
mod new;

type BoxedDropHandler = Box<
    dyn 'static
        + Send
        + Sync
        + Fn(&mut CustomCollections, &EngineInterface, &dyn CustomValue) -> Result<(), LabeledError>,
>;

pub struct HandlebarsPlugin {
    pub collections: RwLock<CustomCollections>,
    drop_handlers: HashMap<TypeId, BoxedDropHandler>,
}

impl Plugin for HandlebarsPlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_owned()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        [
            Box::new(MidNodeCommand {
                name: "handlebars",
                description: "Operate with the Handlebars template engine",
            }) as Box<dyn PluginCommand<Plugin = Self>>,
            Box::new(eval::HandlebarsEvalCommand),
            Box::new(new::HandlebarsNewCommand),
            Box::new(helper::HandlebarsHelperCommand),
        ]
        .into_iter()
        .chain(list::gen_commands())
        .collect()
    }

    fn custom_value_dropped(
        &self,
        engine: &EngineInterface,
        custom_value: Box<dyn CustomValue>,
    ) -> Result<(), nu_protocol::LabeledError> {
        let Some(handler) = self.drop_handlers.get(&custom_value.type_id()) else {
            return Ok(());
        };
        let mut collections = self.collections.write().unwrap();
        let result = handler(&mut collections, engine, custom_value.as_ref());
        if collections.is_empty() {
            engine.set_gc_disabled(false)?;
        }
        result
    }
}

impl HandlebarsPlugin {
    pub fn new() -> Self {
        Self {
            collections: Default::default(),
            drop_handlers: [(
                TypeId::of::<HandlebarsRegistry>(),
                Self::gen_handler(Self::handle_drop_registry),
            )]
            .into_iter()
            .collect(),
        }
    }

    fn gen_handler<C: CustomValue>(
        dlg: fn(&mut CustomCollections, &EngineInterface, &C) -> Result<(), LabeledError>,
    ) -> BoxedDropHandler {
        Box::new(move |collections, engine, custom_value| {
            let concrete_value = custom_value
                .as_any()
                .downcast_ref::<C>()
                .expect("should have been of the correct type");
            dlg(collections, engine, concrete_value)
        })
    }

    fn handle_drop_registry(
        collections: &mut CustomCollections,
        _engine: &EngineInterface,
        registry_reference: &HandlebarsRegistry,
    ) -> Result<(), LabeledError> {
        let Entry::Occupied(mut entry) = collections.registries.entry(*registry_reference.uuid())
        else {
            return Ok(());
        };
        let entry_mut = entry.get_mut();
        if 1 < entry_mut.refcount {
            entry_mut.refcount -= 1;
        } else {
            entry.remove();
        }
        Ok(())
    }
}

struct MidNodeCommand {
    name: &'static str,
    description: &'static str,
}

impl PluginCommand for MidNodeCommand {
    type Plugin = HandlebarsPlugin;

    fn name(&self) -> &str {
        self.name
    }

    fn signature(&self) -> nu_protocol::Signature {
        Signature::build(self.name()).add_help()
    }

    fn description(&self) -> &str {
        self.description
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        engine: &EngineInterface,
        call: &nu_plugin::EvaluatedCall,
        _input: nu_protocol::PipelineData,
    ) -> Result<nu_protocol::PipelineData, nu_protocol::LabeledError> {
        Ok(PipelineData::value(
            Value::string(engine.get_help()?, call.head),
            None,
        ))
    }
}

fn extract_reference_from_input<C: CustomValue + CustomReference>(
    input: &PipelineData,
) -> Result<&C, LabeledError> {
    if let PipelineData::Value(
        Value::Custom {
            val: input,
            internal_span: _,
            ..
        },
        _,
    ) = input
        && let Some(reference) = input.as_any().downcast_ref::<C>()
    {
        Ok(reference)
    } else {
        Err(LabeledError::new(format!("Expected {}", C::NAME))
            .with_label(format!("not {}", C::NAME), input.span().unwrap_or_default()))
    }
}
