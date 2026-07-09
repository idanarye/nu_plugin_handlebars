use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{LabeledError, PipelineData, Signature, SyntaxShape, Type};
use uuid::Uuid;

use crate::custom_value::{
    CustomEntry, CustomReference, RegistryReference, TemplateObject, TemplateReference,
};

use super::{HandlebarsPlugin, compile_template_from_evaluated_call, extract_reference_from_input};

pub struct HandlebarsCompileCommand;

impl PluginCommand for HandlebarsCompileCommand {
    type Plugin = HandlebarsPlugin;

    fn name(&self) -> &str {
        "handlebars compile"
    }

    fn signature(&self) -> nu_protocol::Signature {
        Signature::build(self.name())
            .named(
                "text",
                SyntaxShape::String,
                "The Handlebars template as string",
                Some('t'),
            )
            .named(
                "file",
                SyntaxShape::Filepath,
                "The Handlebars template as file path",
                Some('f'),
            )
            .input_output_type(
                Type::one_of([Type::custom("HandlebarsRegistry"), Type::Nothing]),
                Type::custom("HandlebarsTemplate"),
            )
    }

    fn description(&self) -> &str {
        "Compile an Handlebars template"
    }

    fn run(
        &self,
        plugin: &Self::Plugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let template = compile_template_from_evaluated_call(engine, call)?;

        let mut templates = plugin.collections.templates.write().unwrap();
        let registry_reference = extract_reference_from_input::<RegistryReference>(&input)?;
        if let Some(registry_reference) = registry_reference {
            let mut registries = plugin.collections.registries.write().unwrap();
            let Some(registry_entry) = registries.get_mut(registry_reference.uuid()) else {
                return Err(LabeledError::new("HandlebarsRegistry is not registered")
                    .with_label("not registered", input.span().unwrap_or_default())
                    .with_help("This is probably a bug in the nu_plugin_handlebars"));
            };
            registry_entry.refcount += 1;
        }
        let reference = TemplateReference(Uuid::new_v4());
        templates.insert(
            *reference.uuid(),
            CustomEntry {
                reference: reference.clone(),
                refcount: 1,
                data: TemplateObject {
                    registry: registry_reference.cloned(),
                    template,
                },
            },
        );

        engine.set_gc_disabled(true)?;
        Ok(PipelineData::value(reference.into_value(call.head), None))
    }
}
