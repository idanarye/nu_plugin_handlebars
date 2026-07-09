use std::cell::RefCell;

use handlebars::{
    Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, RenderErrorReason,
    Renderable, ScopedJson, Template,
};
use nu_plugin::EngineInterface;
use nu_protocol::engine::Closure;
use nu_protocol::{IntoValue, LabeledError, Span, Spanned, Value};

use crate::conversions::{json_value_to_nu, nu_value_to_json_value};

thread_local! {
    static ENGINE_INTERFACE: RefCell<Option<EngineInterface>> = const { RefCell::new(None) };
}

pub fn with_engine_in_scope<T>(engine: EngineInterface, dlg: impl FnOnce() -> T) -> T {
    let old_value = ENGINE_INTERFACE.replace(Some(engine));
    let result = dlg();
    ENGINE_INTERFACE.set(old_value);
    result
}

pub fn render_toplevel_handlebars_template(
    engine: EngineInterface,
    template: &Template,
    registry: &Handlebars,
    context: &Context,
    span: Span,
) -> Result<String, LabeledError> {
    let mut rc = RenderContext::new(None);
    with_engine_in_scope(engine, || {
        template.renders(registry, context, &mut rc).map_err(|err| {
            LabeledError::new(err.to_string()).with_label("handlebars rendering", span)
        })
    })
}

handlebars::handlebars_helper!(foo: | | {
    "I am foo"
});

pub struct NuClosureHelper {
    pub closure: Spanned<Closure>,
}

impl HelperDef for NuClosureHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        helper: &Helper<'rc>,
        _registry: &'reg Handlebars<'reg>,
        _context: &'rc Context,
        _render_context: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'rc>, RenderError> {
        if helper.is_block() {
            return Err(
                RenderErrorReason::Other("Block helpers are not supported".to_owned()).into(),
            );
        }
        let positional_arguments = helper
            .params()
            .iter()
            .map(|path_and_json| json_value_to_nu(path_and_json.value()))
            .collect::<Result<_, _>>()?;
        let helper_input: Value = HelperInput {
            hash: helper
                .hash()
                .iter()
                .map(|(name, path_and_json)| {
                    Ok(((*name).to_owned(), json_value_to_nu(path_and_json.value())?))
                })
                .collect::<Result<_, RenderError>>()?,
        }
        .into_value(Span::default());
        let closure_result = ENGINE_INTERFACE.with_borrow(|engine| {
            engine
                .as_ref()
                .expect("Should have one registered")
                .eval_closure(&self.closure, positional_arguments, Some(helper_input))
                .map_err(|err| RenderErrorReason::Other(err.to_string()))
        })?;
        Ok(ScopedJson::Derived(
            nu_value_to_json_value(&closure_result)
                .map_err(|err| RenderErrorReason::Other(err.to_string()))?,
        ))
    }
}

#[derive(IntoValue)]
struct HelperInput {
    // Don't use hashbrown because it cannot be converted to a Nu value
    hash: std::collections::HashMap<String, Value>,
}
