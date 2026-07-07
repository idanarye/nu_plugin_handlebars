use std::sync::RwLock;

use nu_plugin::{Plugin, PluginCommand};
use nu_protocol::{CustomValue, LabeledError, PipelineData, Signature, Value};

use crate::custom_value::{CustomCollections, CustomReference};

mod eval;
mod helper;
mod list;
mod new;

#[derive(Default)]
pub struct HandlebarsPlugin {
    pub collections: RwLock<CustomCollections>,
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
        engine: &nu_plugin::EngineInterface,
        custom_value: Box<dyn nu_protocol::CustomValue>,
    ) -> Result<(), nu_protocol::LabeledError> {
        let collections = self.collections.read().unwrap();
        eprintln!("Dropping {:?}", custom_value);
        if collections.is_empty() {
            engine.set_gc_disabled(false)?;
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
        engine: &nu_plugin::EngineInterface,
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
