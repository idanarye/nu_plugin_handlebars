use handlebars::{Context, Template};
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    LabeledError, ListStream, PipelineData, ShellError, Signature, Span, SyntaxShape, Type, Value,
};

use crate::conversions::nu_value_to_json_value;
use crate::handlebars_tools::{create_handlebars_registry, render_toplevel_handlebars_template};

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
                "helpers",
                SyntaxShape::Record(Default::default()),
                "Define helpers as a record where the values are Nu closures",
                None,
            )
            .named(
                "partials",
                SyntaxShape::Record(Default::default()),
                "Define partials as a record where the values are Handlebars partial templates as strings (not file paths)",
                None,
            )
            .input_output_types(vec![
                (Type::Nothing, Type::Closure),
                (Type::record(), Type::String),
                (Type::list(Type::record()), Type::list(Type::String)),
            ])
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
        let template_text = &call.positional[0];
        let template = Template::compile(template_text.as_str()?).map_err(|err| {
            LabeledError::new(err.to_string())
                .with_label("handlebars syntax error", template_text.span())
        })?;
        let hb = create_handlebars_registry(
            engine,
            call.get_flag("partials")?.unwrap_or_default(),
            call.get_flag("helpers")?.unwrap_or_default(),
        )?;

        let input_span = input.span();

        Ok(match input {
            PipelineData::Empty => {
                Err(LabeledError::new("Missing input").with_label("needs input", call.head))?
            }
            PipelineData::Value(Value::List { vals, .. }, pipeline_metadata) => {
                let call_span = call.head;
                PipelineData::ListStream(
                    ListStream::new(
                        vals.into_iter().map(move |value| {
                            let context = match nu_value_to_json_value(&value) {
                                Ok(json_value) => Context::from(json_value),
                                Err(err) => {
                                    return Value::error(
                                        ShellError::LabeledError(Box::new(err)),
                                        value.span(),
                                    );
                                }
                            };
                            match render_toplevel_handlebars_template(
                                &template, &hb, &context, call_span,
                            ) {
                                Ok(ok) => Value::string(ok, Span::default()),
                                Err(err) => Value::error(
                                    ShellError::LabeledError(Box::new(err)),
                                    value.span(),
                                ),
                            }
                        }),
                        input_span.expect("piepline value should have a span"),
                        engine.signals().clone(),
                    ),
                    pipeline_metadata,
                )
            }
            PipelineData::Value(value, pipeline_metadata) => {
                let context = Context::from(nu_value_to_json_value(&value)?);
                let rendered = Value::string(
                    render_toplevel_handlebars_template(&template, &hb, &context, call.head)?,
                    Span::default(),
                );
                PipelineData::Value(rendered, pipeline_metadata)
            }
            PipelineData::ListStream(list_stream, pipeline_metadata) => PipelineData::ListStream(
                list_stream.map(move |value| {
                    let context = match nu_value_to_json_value(&value) {
                        Ok(json_value) => Context::from(json_value),
                        Err(err) => {
                            return Value::error(
                                ShellError::LabeledError(Box::new(err)),
                                value.span(),
                            );
                        }
                    };
                    match render_toplevel_handlebars_template(
                        &template,
                        &hb,
                        &context,
                        value.span(),
                    ) {
                        Ok(ok) => Value::string(ok, Span::default()),
                        Err(err) => {
                            Value::error(ShellError::LabeledError(Box::new(err)), value.span())
                        }
                    }
                }),
                pipeline_metadata,
            ),
            PipelineData::ByteStream(_byte_stream, _pipeline_metadata) => {
                return Err(ShellError::PipelineMismatch {
                    exp_input_type: "Record or stream/list of records".to_owned(),
                    dst_span: call.head,
                    src_span: _byte_stream.span(),
                }
                .into());
            }
        })
    }
}
