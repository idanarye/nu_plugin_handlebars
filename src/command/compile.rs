use handlebars::Template;
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{LabeledError, PipelineData, Signature, Spanned, SyntaxShape, Type};
use uuid::Uuid;

use crate::custom_value::{
    CustomEntry, CustomReference, RegistryReference, TemplateObject, TemplateReference,
};

use super::{HandlebarsPlugin, extract_reference_from_input};

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
        "Render using an Handlebars template"
    }

    fn run(
        &self,
        plugin: &Self::Plugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let template = match (
            call.get_flag::<Spanned<String>>("text")?,
            call.get_flag::<Spanned<String>>("file")?,
        ) {
            (None, None) => {
                return Err(LabeledError::new("Need either `--text` or `--file`")
                    .with_label("for this", call.head));
            }
            (Some(text), Some(file)) => {
                return Err({
                    LabeledError::new("Do not provide both `--text` and `--file`")
                        .with_label("cannot have both", text.span)
                        .with_label("cannot have both", file.span)
                });
            }
            (Some(text), None) => Template::compile(&text.item)
                .map_err(|err| LabeledError::new(err.to_string()).with_label("here", text.span))?,
            (None, Some(file)) => {
                let full_path = std::path::Path::new(&engine.get_current_dir()?).join(&file.item);
                Template::compile_with_name(
                    std::fs::read_to_string(full_path).map_err(|err| {
                        LabeledError::new(err.to_string()).with_label("this", file.span)
                    })?,
                    file.item,
                )
                .map_err(|err| {
                    LabeledError::new(err.to_string()).with_label("this file", file.span)
                })?
            }
        };

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
