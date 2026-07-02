use std::collections::HashMap;

use handlebars::{
    Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, RenderErrorReason,
    Renderable, ScopedJson, Template,
};
use nu_plugin::EngineInterface;
use nu_protocol::engine::Closure;
use nu_protocol::{LabeledError, Span, Spanned};

use crate::conversions::{json_value_to_nu, nu_value_to_json_value};

pub fn render_toplevel_handlebars_template(
    template: &Template,
    registry: &Handlebars,
    context: &Context,
    span: Span,
) -> Result<String, LabeledError> {
    let mut rc = RenderContext::new(None);
    template
        .renders(registry, context, &mut rc)
        .map_err(|err| LabeledError::new(err.to_string()).with_label("handlebars rendering", span))
}

pub fn create_handlebars_registry<'a>(
    engine: &'_ EngineInterface,
    partials: HashMap<String, Spanned<String>>,
    helpers: HashMap<String, Spanned<Closure>>,
) -> Result<Handlebars<'a>, LabeledError> {
    let mut hb = Handlebars::new();

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
        helper: &Helper<'rc>,
        _registry: &'reg Handlebars<'reg>,
        _context: &'rc Context,
        _render_context: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'rc>, RenderError> {
        if helper.is_block() {
            return Err(RenderErrorReason::Other("Block helpers are not supported".to_owned()).into());
        }
        let closure_result = self
            .engine
            .eval_closure(
                &self.closure,
                helper
                    .params()
                    .iter()
                    .map(|path_and_json| json_value_to_nu(path_and_json.value()))
                    .collect::<Result<_, _>>()?,
                None,
            )
            .map_err(|err| RenderErrorReason::Other(err.to_string()))?;
        Ok(ScopedJson::Derived(
            nu_value_to_json_value(&closure_result)
                .map_err(|err| RenderErrorReason::Other(err.to_string()))?,
        ))
    }
}
