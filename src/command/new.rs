use handlebars::Handlebars;
use nu_plugin::PluginCommand;
use nu_protocol::{PipelineData, Signature, Type};
use uuid::Uuid;

use crate::custom_value::{CustomEntry, CustomReference, HandlebarsRegistry};

use super::HandlebarsPlugin;

pub struct HandlebarsNewCommand;

impl PluginCommand for HandlebarsNewCommand {
    type Plugin = HandlebarsPlugin;

    fn name(&self) -> &str {
        "handlebars new"
    }

    fn signature(&self) -> nu_protocol::Signature {
        Signature::build(self.name())
            .input_output_type(Type::Nothing, Type::custom("HandlebarsRegistry"))
    }

    fn description(&self) -> &str {
        "Initialize a new Handlebars registry"
    }

    fn run(
        &self,
        plugin: &Self::Plugin,
        engine: &nu_plugin::EngineInterface,
        call: &nu_plugin::EvaluatedCall,
        _input: PipelineData,
    ) -> Result<PipelineData, nu_protocol::LabeledError> {
        let reference = HandlebarsRegistry(Uuid::new_v4());
        plugin.collections.write().unwrap().registries.insert(
            *reference.uuid(),
            CustomEntry {
                reference: reference.clone(),
                refcount: 0,
                data: Handlebars::new(),
            },
        );
        engine.set_gc_disabled(true)?;
        Ok(PipelineData::value(reference.into_value(call.head), None))
    }
}
