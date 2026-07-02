use nu_plugin::{
    EngineInterface, EvaluatedCall, MsgPackSerializer, Plugin, PluginCommand, serve_plugin,
};
use nu_protocol::{LabeledError, PipelineData, Signature, SyntaxShape, Type};

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
        plugin: &Self::Plugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        Ok(PipelineData::empty())
    }
}

fn main() {
    serve_plugin(&HandlebarsPlugin, MsgPackSerializer);
}
