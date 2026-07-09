use std::any::TypeId;

use hashbrown::HashMap;
use hashbrown::hash_map::Entry;
use nu_plugin::{EngineInterface, Plugin, PluginCommand};
use nu_protocol::{CustomValue, LabeledError, PipelineData, Signature, Value};

use crate::custom_value::{CustomCollections, CustomReference, RegistryReference};

mod compile;
mod eval;
mod helper;
mod list;
mod new;

type BoxedDropHandler = Box<
    dyn 'static
        + Send
        + Sync
        + Fn(&CustomCollections, &EngineInterface, &dyn CustomValue) -> Result<(), LabeledError>,
>;

pub struct HandlebarsPlugin {
    pub collections: CustomCollections,
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
            Box::new(new::HandlebarsNewCommand),
            Box::new(helper::HandlebarsHelperCommand),
            Box::new(eval::HandlebarsEvalCommand),
            Box::new(compile::HandlebarsCompileCommand),
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
        let result = handler(&self.collections, engine, custom_value.as_ref());
        if self.collections.is_empty() {
            engine.set_gc_disabled(false)?;
        }
        result
    }
}

impl HandlebarsPlugin {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            collections: Default::default(),
            drop_handlers: [(
                TypeId::of::<RegistryReference>(),
                Self::gen_handler(Self::handle_drop_registry),
            )]
            .into_iter()
            .collect(),
        }
    }

    fn gen_handler<C: CustomValue>(
        dlg: fn(&CustomCollections, &EngineInterface, &C) -> Result<(), LabeledError>,
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
        collections: &CustomCollections,
        _engine: &EngineInterface,
        registry_reference: &RegistryReference,
    ) -> Result<(), LabeledError> {
        let mut registries = collections.registries.write().unwrap();
        let Entry::Occupied(mut entry) = registries.entry(*registry_reference.uuid()) else {
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
) -> Result<Option<&C>, LabeledError> {
    match input {
        PipelineData::Empty => Ok(None),
        PipelineData::Value(
            Value::Custom {
                val: input,
                internal_span: _,
                ..
            },
            _,
        ) if let Some(reference) = input.as_any().downcast_ref::<C>() => Ok(Some(reference)),
        _ => Err(LabeledError::new(format!("Expected {}", C::NAME))
            .with_label(format!("not {}", C::NAME), input.span().unwrap_or_default())),
    }
}
