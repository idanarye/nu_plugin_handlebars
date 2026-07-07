use nu_plugin::PluginCommand;
use nu_protocol::engine::Closure;
use nu_protocol::{LabeledError, PipelineData, Signature, Spanned, SyntaxShape, Type};

use crate::command::extract_reference_from_input;
use crate::custom_value::{CustomReference, HandlebarsRegistry};
use crate::handlebars_tools::NuClosureHelper;

use super::HandlebarsPlugin;

pub struct HandlebarsHelperCommand;

impl PluginCommand for HandlebarsHelperCommand {
    type Plugin = HandlebarsPlugin;

    fn name(&self) -> &str {
        "handlebars helper"
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
                "The name used to invoke the helper",
            )
            .required(
                "body",
                SyntaxShape::Closure(None),
                "A closure implementing the helper",
            )
    }

    fn description(&self) -> &str {
        "Register an Handlebars helper"
    }

    fn run(
        &self,
        plugin: &Self::Plugin,
        engine: &nu_plugin::EngineInterface,
        call: &nu_plugin::EvaluatedCall,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let registry_reference: &HandlebarsRegistry = extract_reference_from_input(&input)?;

        let name: String = call.req(0)?;
        let closure: Spanned<Closure> = call.req(1)?;

        let mut collections = plugin.collections.write().unwrap();
        let Some(registry) = collections.registries.get_mut(registry_reference.uuid()) else {
            return Err(LabeledError::new("HandlebarsRegistry is not registered")
                .with_label("not registered", input.span().unwrap_or_default())
                .with_help("This is probably a bug in the nu_plugin_handlebars"));
        };
        registry.data.register_helper(
            &name,
            Box::new(NuClosureHelper {
                engine: engine.clone(),
                closure,
            }),
        );
        Ok(input)
    }
}
