use std::any::TypeId;
use std::path::PathBuf;

use handlebars::Template;
use hashbrown::HashMap;
use hashbrown::hash_map::Entry;
use nu_plugin::{EngineInterface, EvaluatedCall, Plugin, PluginCommand};
use nu_protocol::{CustomValue, LabeledError, PipelineData, Signature, Spanned, Value};

use crate::custom_value::{
    CustomCollections, CustomReference, RegistryReference, TemplateReference,
};

mod compile;
mod eval;
mod helper;
mod new;
mod partial;
mod render;

type BoxedDropHandler = Box<
    dyn 'static
        + Send
        + Sync
        + Fn(&CustomCollections, &EngineInterface, &dyn CustomValue) -> Result<(), LabeledError>,
>;

pub struct HandlebarsPlugin {
    pub collections: CustomCollections,
    drop_handlers: HashMap<TypeId, BoxedDropHandler>,
}

impl Plugin for HandlebarsPlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_owned()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        [
            Box::new(MidNodeCommand {
                name: "handlebars",
                description: "Operate with the Handlebars template engine",
            }) as Box<dyn PluginCommand<Plugin = Self>>,
            Box::new(new::HandlebarsNewCommand),
            Box::new(helper::HandlebarsHelperCommand),
            Box::new(partial::HandlebarsPartialCommand),
            Box::new(eval::HandlebarsEvalCommand),
            Box::new(compile::HandlebarsCompileCommand),
            Box::new(render::HandlebarsRenderCommand),
        ]
        .into_iter()
        .collect()
    }

    fn custom_value_dropped(
        &self,
        engine: &EngineInterface,
        custom_value: Box<dyn CustomValue>,
    ) -> Result<(), nu_protocol::LabeledError> {
        let Some(handler) = self.drop_handlers.get(&custom_value.type_id()) else {
            return Ok(());
        };
        let result = handler(&self.collections, engine, custom_value.as_ref());
        if self.collections.is_empty() {
            engine.set_gc_disabled(false)?;
        }
        result
    }
}

impl HandlebarsPlugin {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            collections: Default::default(),
            drop_handlers: [
                Self::gen_handler(Self::handle_drop_registry),
                Self::gen_handler(Self::handle_drop_template),
            ]
            .into_iter()
            .collect(),
        }
    }

    fn gen_handler<C: CustomValue>(
        dlg: fn(&CustomCollections, &EngineInterface, &C) -> Result<(), LabeledError>,
    ) -> (TypeId, BoxedDropHandler) {
        (
            TypeId::of::<C>(),
            Box::new(move |collections, engine, custom_value| {
                let concrete_value = custom_value
                    .as_any()
                    .downcast_ref::<C>()
                    .expect("should have been of the correct type");
                dlg(collections, engine, concrete_value)
            }),
        )
    }

    fn handle_drop_registry(
        collections: &CustomCollections,
        _engine: &EngineInterface,
        registry_reference: &RegistryReference,
    ) -> Result<(), LabeledError> {
        let mut registries = collections.registries.write().unwrap();
        let Entry::Occupied(mut entry) = registries.entry(*registry_reference.uuid()) else {
            return Ok(());
        };
        let entry_mut = entry.get_mut();
        if 1 < entry_mut.refcount {
            entry_mut.refcount -= 1;
        } else {
            entry.remove();
        }
        Ok(())
    }

    fn handle_drop_template(
        collections: &CustomCollections,
        _engine: &EngineInterface,
        template_reference: &TemplateReference,
    ) -> Result<(), LabeledError> {
        let mut templates = collections.templates.write().unwrap();
        let Entry::Occupied(mut entry) = templates.entry(*template_reference.uuid()) else {
            return Ok(());
        };
        let entry_mut = entry.get_mut();
        if 1 < entry_mut.refcount {
            entry_mut.refcount -= 1;
        } else {
            if let Some(registry_reference) = &entry_mut.data.registry {
                Self::handle_drop_registry(collections, _engine, registry_reference)?;
            }

            entry.remove();
        }
        Ok(())
    }
}

struct MidNodeCommand {
    name: &'static str,
    description: &'static str,
}

impl PluginCommand for MidNodeCommand {
    type Plugin = HandlebarsPlugin;

    fn name(&self) -> &str {
        self.name
    }

    fn signature(&self) -> nu_protocol::Signature {
        Signature::build(self.name()).add_help()
    }

    fn description(&self) -> &str {
        self.description
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        engine: &EngineInterface,
        call: &nu_plugin::EvaluatedCall,
        _input: nu_protocol::PipelineData,
    ) -> Result<nu_protocol::PipelineData, nu_protocol::LabeledError> {
        Ok(PipelineData::value(
            Value::string(engine.get_help()?, call.head),
            None,
        ))
    }
}

fn extract_reference_from_input<C: CustomValue + CustomReference>(
    input: &PipelineData,
) -> Result<Option<&C>, LabeledError> {
    match input {
        PipelineData::Empty => Ok(None),
        PipelineData::Value(
            Value::Custom {
                val: input,
                internal_span: _,
                ..
            },
            _,
        ) if let Some(reference) = input.as_any().downcast_ref::<C>() => Ok(Some(reference)),
        _ => Err(LabeledError::new(format!("Expected {}", C::NAME))
            .with_label(format!("not {}", C::NAME), input.span().unwrap_or_default())),
    }
}

pub fn compile_template_from_evaluated_call(
    engine: &EngineInterface,
    call: &EvaluatedCall,
) -> Result<Template, LabeledError> {
    Ok(
        match (
            call.get_flag::<Spanned<String>>("text")?,
            call.get_flag::<Spanned<PathBuf>>("file")?,
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
                    file.item.to_string_lossy().into_owned(),
                )
                .map_err(|err| {
                    LabeledError::new(err.to_string()).with_label("this file", file.span)
                })?
            }
        },
    )
}
