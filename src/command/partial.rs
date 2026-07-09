use nu_plugin::PluginCommand;
use nu_protocol::{LabeledError, PipelineData, Signature, SyntaxShape, Type};

use crate::command::extract_reference_from_input;
use crate::custom_value::{CustomReference, RegistryReference};

use super::{HandlebarsPlugin, compile_template_from_evaluated_call};

pub struct HandlebarsPartialCommand;

impl PluginCommand for HandlebarsPartialCommand {
    type Plugin = HandlebarsPlugin;

    fn name(&self) -> &str {
        "handlebars partial"
    }

    fn signature(&self) -> nu_protocol::Signature {
        Signature::build(self.name())
            .add_help()
            .input_output_type(
                Type::custom("HandlebarsRegistry"),
                Type::custom("HandlebarsRegistry"),
            )
            .required(
                "name",
                SyntaxShape::String,
                "The name used to invoke the partial",
            )
            .named(
                "text",
                SyntaxShape::String,
                "The Handlebars partial as string",
                Some('t'),
            )
            .named(
                "file",
                SyntaxShape::Filepath,
                "The Handlebars partial as file path",
                Some('f'),
            )
    }

    fn description(&self) -> &str {
        "Register an Handlebars partial"
    }

    fn run(
        &self,
        plugin: &Self::Plugin,
        engine: &nu_plugin::EngineInterface,
        call: &nu_plugin::EvaluatedCall,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let registry_reference: &RegistryReference =
            extract_reference_from_input(&input)?.expect("Signature should prevent this");

        let name: String = call.req(0)?;
        let template = compile_template_from_evaluated_call(engine, call)?;

        let mut registries = plugin.collections.registries.write().unwrap();
        let Some(registry_entry) = registries.get_mut(registry_reference.uuid()) else {
            return Err(LabeledError::new("HandlebarsRegistry is not registered")
                .with_label("not registered", input.span().unwrap_or_default())
                .with_help("This is probably a bug in the nu_plugin_handlebars"));
        };
        registry_entry.data.register_template(&name, template);
        registry_entry.refcount += 1;
        Ok(input)
    }
}
