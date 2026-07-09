use handlebars::{Handlebars, Template};
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{LabeledError, PipelineData, Signature, Span, Spanned, SyntaxShape, Type, Value};

use crate::conversions::nu_input_to_handlebars_context;
use crate::custom_value::{CustomReference, RegistryReference};
use crate::handlebars_tools::render_toplevel_handlebars_template;

use super::HandlebarsPlugin;

pub struct HandlebarsEvalCommand;

impl PluginCommand for HandlebarsEvalCommand {
    type Plugin = HandlebarsPlugin;

    fn name(&self) -> &str {
        "handlebars eval"
    }

    fn signature(&self) -> nu_protocol::Signature {
        Signature::build(self.name())
            .required("template", SyntaxShape::String, "The Handlebars template as string (not file path)")
            .named(
                "with",
                SyntaxShape::Any,
                "Evaluate the template using an Handlebars registry (can be created with `handlebars new`)",
                None
            )
            .input_output_type(Type::Any, Type::String)
    }

    fn description(&self) -> &str {
        "Render using an Handlebars template provided as string"
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let context = nu_input_to_handlebars_context(input, call.head)?;

        let template_text = &call.positional[0];
        let template = Template::compile(template_text.as_str()?).map_err(|err| {
            LabeledError::new(err.to_string())
                .with_label("handlebars syntax error", template_text.span())
        })?;
        let hb_memory_slot;
        let registry_lock;
        let hb = if let Some(registry_reference) =
            call.get_flag::<Spanned<RegistryReference>>("with")?
        {
            registry_lock = _plugin.collections.registries.read().unwrap();
            let Some(registry_entry) = registry_lock.get(registry_reference.item.uuid()) else {
                return Err(LabeledError::new("HandlebarsRegistry is not registered")
                    .with_label("not registered", registry_reference.span)
                    .with_help("This is probably a bug in the nu_plugin_handlebars"));
            };
            &registry_entry.data
        } else {
            hb_memory_slot = Handlebars::new();
            &hb_memory_slot
        };

        let rendered = Value::string(
            render_toplevel_handlebars_template(
                engine.clone(),
                &template,
                hb,
                &context,
                call.head,
            )?,
            Span::default(),
        );
        Ok(PipelineData::Value(rendered, None))
    }
}
