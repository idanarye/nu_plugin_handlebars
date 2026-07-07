use nu_plugin::PluginCommand;
use nu_protocol::{LabeledError, PipelineData, Signature, Type, Value};

use crate::custom_value::{CustomEntry, CustomReference, HandlebarsRegistry};

use super::{HandlebarsPlugin, MidNodeCommand, extract_reference_from_input};

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
        mut input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let registry_reference: &HandlebarsRegistry = extract_reference_from_input(&input)?;
        let collections = plugin.collections.read().unwrap();
        let Some(entry) = collections.registries.get(registry_reference.uuid()) else {
            return Err(LabeledError::new("HandlebarsRegistry is not registered")
                .with_label("not registered", input.span().unwrap_or_default())
                .with_help("This is probably a bug in the nu_plugin_handlebars"));
        };
        Ok(PipelineData::Value(
            Value::list((self.extractor)(entry), input.span().unwrap_or_default()),
            input.take_metadata(),
        ))
    }
}
