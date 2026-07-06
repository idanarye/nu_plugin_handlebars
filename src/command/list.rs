use nu_plugin::PluginCommand;
use nu_protocol::{LabeledError, PipelineData, Signature, Type, Value};

use crate::custom_value::{CustomEntry, CustomReference, HandlebarsRegistry};

use super::{HandlebarsPlugin, MidNodeCommand};

pub struct HandlebarsListCommand<F>
where
    F: 'static + Send + Sync + Fn(&CustomEntry<HandlebarsRegistry>) -> Vec<Value>,
{
    name: &'static str,
    description: &'static str,
    extractor: F,
}

pub fn gen_commands() -> impl Iterator<Item = Box<dyn PluginCommand<Plugin = HandlebarsPlugin>>> {
    [
        Box::new(MidNodeCommand {
            name: "handlebars list",
            description: "Retrieve information about Handlebars registry",
        }) as Box<dyn PluginCommand<Plugin = HandlebarsPlugin>>,
        Box::new(HandlebarsListCommand {
            name: "handlebars list helpers",
            description: "List the names of all the helpers registered in the registry",
            extractor: |_entry| [].into(),
        }),
        Box::new(HandlebarsListCommand {
            name: "handlebars list partials",
            description: "List the names of all the partials registered in the registry",
            extractor: |entry| {
                entry
                    .data
                    .get_templates()
                    .keys()
                    .map(|n| Value::string(n, Default::default()))
                    .collect()
            },
        }),
    ]
    .into_iter()
}

impl<F> PluginCommand for HandlebarsListCommand<F>
where
    F: 'static + Send + Sync + Fn(&CustomEntry<HandlebarsRegistry>) -> Vec<Value>,
{
    type Plugin = HandlebarsPlugin;

    fn name(&self) -> &str {
        self.name
    }

    fn signature(&self) -> nu_protocol::Signature {
        Signature::build(self.name())
            .add_help()
            .input_output_type(Type::custom("HandlebarsRegistry"), Type::list(Type::String))
    }

    fn description(&self) -> &str {
        self.description
    }

    fn run(
        &self,
        plugin: &Self::Plugin,
        _engine: &nu_plugin::EngineInterface,
        _call: &nu_plugin::EvaluatedCall,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let PipelineData::Value(
            Value::Custom {
                val: input,
                internal_span: input_span,
                ..
            },
            pipeline_metadata,
        ) = input
        else {
            return Err(LabeledError::new("Expected an HandlebarsRegistry")
                .with_label("not HandlebarsRegistry", input.span().unwrap_or_default()));
        };
        let Some(registry_reference) = input.as_any().downcast_ref::<HandlebarsRegistry>() else {
            return Err(LabeledError::new("Expected an HandlebarsRegistry")
                .with_label("not HandlebarsRegistry", input_span));
        };
        let collections = plugin.collections.read().unwrap();
        let Some(entry) = collections.registries.get(registry_reference.uuid()) else {
            return Err(LabeledError::new("HandlebarsRegistry is not registered")
                .with_label("not registered", input_span)
                .with_help("This is probably a bug in the nu_plugin_handlebars"));
        };
        Ok(PipelineData::Value(
            Value::list((self.extractor)(&entry), input_span),
            pipeline_metadata,
        ))
        // Ok(PipelineData::Value(
        // Value::list(
        // entry
        // .data
        // .get_templates()
        // .keys()
        // .map(|v| Value::string(v, Default::default()))
        // .collect(),
        // input_span,
        // ),
        // pipeline_metadata,
        // ))
    }
}
