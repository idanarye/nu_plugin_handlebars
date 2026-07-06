use std::sync::Mutex;

use nu_plugin::{Plugin, PluginCommand};
use nu_protocol::{PipelineData, Signature, Value};

use crate::custom_value::CustomCollections;

mod eval;
mod new;

#[derive(Default)]
pub struct HandlebarsPlugin {
    pub collections: Mutex<CustomCollections>,
}

impl Plugin for HandlebarsPlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_owned()
    }

    fn commands(&self) -> Vec<Box<dyn nu_plugin::PluginCommand<Plugin = Self>>> {
        vec![
            Box::new(MidNodeCommand {
                name: "handlebars",
                description: "Operate with the Handlebars template engine",
            }),
            Box::new(eval::HandlebarsEvalCommand),
            Box::new(new::HandlebarsNewCommand),
        ]
    }

    fn custom_value_dropped(
        &self,
        _engine: &nu_plugin::EngineInterface,
        custom_value: Box<dyn nu_protocol::CustomValue>,
    ) -> Result<(), nu_protocol::LabeledError> {
        eprintln!("Dropping {:?}", custom_value);
        println!("{:?}", self.collections.lock());
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
