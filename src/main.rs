use handlebars::{Context, Handlebars, Template};
use nu_plugin::{
    EngineInterface, EvaluatedCall, MsgPackSerializer, Plugin, PluginCommand, serve_plugin,
};
use nu_plugin_handlebars::conversions::nu_value_to_json_value;
use nu_protocol::{
    LabeledError, ListStream, PipelineData, ShellError, Signature, Span, SyntaxShape, Type, Value,
};

pub struct HandlebarsPlugin;

impl Plugin for HandlebarsPlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_owned()
    }

    fn commands(&self) -> Vec<Box<dyn nu_plugin::PluginCommand<Plugin = Self>>> {
        vec![Box::new(HandlebarsCommand)]
    }
}

pub struct HandlebarsCommand;

impl PluginCommand for HandlebarsCommand {
    type Plugin = HandlebarsPlugin;

    fn name(&self) -> &str {
        "handlebars"
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
        let mut hb = Handlebars::new();

        let template_text = call.positional[0].as_str()?;
        let template =
            Template::compile(template_text).map_err(|err| LabeledError::new(err.to_string()))?;
        hb.register_template("", template);

        let input_span = input.span();

        Ok(match input {
            PipelineData::Empty => {
                Err(LabeledError::new("Missing input").with_label("needs input", call.head))?
            }
            PipelineData::Value(Value::List { vals, .. }, pipeline_metadata) => {
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
                            Value::string(
                                hb.render_with_context("", &context).unwrap(),
                                Span::default(),
                            )
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
                    hb.render_with_context("", &context).unwrap(),
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
                    Value::string(
                        hb.render_with_context("", &context).unwrap(),
                        Span::default(),
                    )
                }),
                pipeline_metadata,
            ),
            PipelineData::ByteStream(_byte_stream, _pipeline_metadata) => todo!(),
        })
    }
}

fn main() {
    serve_plugin(&HandlebarsPlugin, MsgPackSerializer);
}
