use std::collections::HashMap;

use handlebars::Handlebars;
use nu_protocol::{LabeledError, Spanned, Value};

pub fn create_handlebars_template<'a>(
    template_text: &Value,
    partials: HashMap<String, Spanned<String>>,
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

    Ok(hb)
}
