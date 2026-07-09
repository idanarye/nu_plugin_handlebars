use handlebars::Template;
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{LabeledError, PipelineData, Signature, Spanned, SyntaxShape, Type};

use crate::custom_value::RegistryReference;

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
        _plugin: &Self::Plugin,
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
            (Some(text), None) => Template::compile(&text.item),
            (None, Some(file)) => {
                let full_path = std::path::Path::new(&engine.get_current_dir()?).join(&file.item);
                Template::compile_with_name(
                    std::fs::read_to_string(full_path).map_err(|err| {
                        LabeledError::new(err.to_string()).with_label("this", file.span)
                    })?,
                    file.item,
                )
            }
        };

        let registry_reference: Option<&RegistryReference> = extract_reference_from_input(&input)?;
        eprintln!("{registry_reference:?}\n{template:?}");

        Ok(PipelineData::empty())
    }
}
