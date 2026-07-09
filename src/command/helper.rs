use nu_plugin::PluginCommand;
use nu_protocol::engine::Closure;
use nu_protocol::{LabeledError, PipelineData, Signature, Spanned, SyntaxShape, Type};

use crate::command::extract_reference_from_input;
use crate::custom_value::{CustomReference, RegistryReference};
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

    fn extra_description(&self) -> &str {
        dedent::dedent!(
            r#"
            The closure implementing the helper receives the helper position arguments as
            positional arguments.

            The `$in` passed to the closure impelemnting the helper has the following fields:

              * `$in.hash` - a record containing the hash (keyword) arguments passed to the helper.

            NOTE: Block helper are not yet supported.
            "#
        )
    }

    fn run(
        &self,
        plugin: &Self::Plugin,
        _engine: &nu_plugin::EngineInterface,
        call: &nu_plugin::EvaluatedCall,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let registry_reference: &RegistryReference =
            extract_reference_from_input(&input)?.expect("Signature should prevent this");

        let name: String = call.req(0)?;
        let closure: Spanned<Closure> = call.req(1)?;

        let mut registries = plugin.collections.registries.write().unwrap();
        let Some(registry_entry) = registries.get_mut(registry_reference.uuid()) else {
            return Err(LabeledError::new("HandlebarsRegistry is not registered")
                .with_label("not registered", input.span().unwrap_or_default())
                .with_help("This is probably a bug in the nu_plugin_handlebars"));
        };
        registry_entry
            .data
            .register_helper(&name, Box::new(NuClosureHelper { closure }));
        registry_entry.refcount += 1;
        Ok(input)
    }
}
