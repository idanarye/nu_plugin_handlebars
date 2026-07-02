use std::collections::HashMap;

use handlebars::{
    Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, RenderErrorReason,
    ScopedJson,
};
use nu_plugin::EngineInterface;
use nu_protocol::engine::Closure;
use nu_protocol::{LabeledError, Spanned, Value};

use crate::conversions::nu_value_to_json_value;

pub fn create_handlebars_template<'a>(
    engine: &'_ EngineInterface,
    template_text: &Value,
    partials: HashMap<String, Spanned<String>>,
    helpers: HashMap<String, Spanned<Closure>>,
) -> Result<Handlebars<'a>, LabeledError> {
    let mut hb = Handlebars::new();

    hb.register_template_string("", template_text.as_str()?)
        .map_err(|err| {
            LabeledError::new(err.to_string())
                .with_label("handlebars syntax error", template_text.span())
        })?;

    for (name, text) in partials.into_iter() {
        hb.register_partial(&name, text.item).map_err(|err| {
            LabeledError::new(err.to_string()).with_label("handlebars syntax error", text.span)
        })?;
    }

    for (name, closure) in helpers.into_iter() {
        hb.register_helper(
            &name,
            Box::new(NuClosureHelper {
                engine: engine.clone(),
                closure,
            }),
        );
    }

    Ok(hb)
}

handlebars::handlebars_helper!(foo: | | {
    "I am foo"
});

struct NuClosureHelper {
    engine: EngineInterface,
    closure: Spanned<Closure>,
}

impl HelperDef for NuClosureHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        _helper: &Helper<'rc>,
        _registry: &'reg Handlebars<'reg>,
        _context: &'rc Context,
        _render_context: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'rc>, RenderError> {
        let closure_result = self
            .engine
            .eval_closure(&self.closure, vec![], None)
            .map_err(|err| RenderErrorReason::Other(err.to_string()))?;
        Ok(ScopedJson::Derived(
            nu_value_to_json_value(&closure_result)
                .map_err(|err| RenderErrorReason::Other(err.to_string()))?,
        ))
    }
}
