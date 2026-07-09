use handlebars::Handlebars;
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{LabeledError, PipelineData, Signature, Spanned, SyntaxShape, Type, Value};

use crate::conversions::nu_input_to_handlebars_context;
use crate::custom_value::{CustomReference, TemplateReference};
use crate::handlebars_tools::render_toplevel_handlebars_template;

use super::HandlebarsPlugin;

pub struct HandlebarsRenderCommand;

impl PluginCommand for HandlebarsRenderCommand {
    type Plugin = HandlebarsPlugin;

    fn name(&self) -> &str {
        "handlebars render"
    }

    fn signature(&self) -> nu_protocol::Signature {
        Signature::build(self.name())
            .required(
                "template",
                SyntaxShape::Any,
                "An Handlebars template - obtainable via `handlebars compile`",
            )
            .input_output_type(Type::Any, Type::String)
    }

    fn description(&self) -> &str {
        "Render using a compiled Handlebars template (obtainable via `handlebars compile`)"
    }

    fn run(
        &self,
        plugin: &Self::Plugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let context = nu_input_to_handlebars_context(input, call.head)?;

        let template_reference: Spanned<TemplateReference> = call.req(0)?;
        let templates = plugin.collections.templates.read().unwrap();

        let Some(template_entry) = templates.get(template_reference.item.uuid()) else {
            return Err(LabeledError::new("HandlebarsTemplate is not registered")
                .with_label("not registered", template_reference.span)
                .with_help("This is probably a bug in the nu_plugin_handlebars plugin"));
        };

        let registries;
        let blank_template;
        let registry = if let Some(registry_reference) = template_entry.data.registry.as_ref() {
            registries = plugin.collections.registries.read().unwrap();
            let Some(registry_entry) = registries.get(registry_reference.uuid()) else {
                return Err(LabeledError::new(
                    "HandlebarsRegistry associated with template is not registered",
                )
                .with_label(
                    "associated registry not registered",
                    template_reference.span,
                )
                .with_help("This is probably a bug in the nu_plugin_handlebars plugin"));
            };
            &registry_entry.data
        } else {
            blank_template = Handlebars::new();
            &blank_template
        };

        let rendered = render_toplevel_handlebars_template(
            engine.clone(),
            &template_entry.data.template,
            registry,
            &context,
            call.head,
        )?;

        Ok(PipelineData::value(
            Value::string(rendered, Default::default()),
            None,
        ))
    }
}
