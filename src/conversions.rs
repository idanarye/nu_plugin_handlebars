use handlebars::{RenderError, RenderErrorReason};
use nu_protocol::{LabeledError, Span, Value};
use serde_json::{Number as JsonNumber, Value as JsonValue};

pub fn nu_value_to_json_value(nu_value: &Value) -> Result<JsonValue, LabeledError> {
    Ok(match nu_value {
        Value::Bool { val, .. } => JsonValue::Bool(*val),
        Value::Int { val, .. } => JsonValue::Number((*val).into()),
        Value::Float { val, .. } => JsonValue::Number(JsonNumber::from_f64(*val).unwrap()),
        Value::String { val, .. } => JsonValue::String(val.clone()),
        Value::Record { val, .. } => JsonValue::Object(
            val.iter()
                .map(|(k, v)| Ok((k.to_owned(), nu_value_to_json_value(v)?)))
                .collect::<Result<_, LabeledError>>()?,
        ),
        Value::List { vals, .. } => JsonValue::Array(
            vals.iter()
                .map(nu_value_to_json_value)
                .collect::<Result<_, _>>()?,
        ),
        Value::Nothing { .. } => JsonValue::Null,

        Value::Glob { .. }
        | Value::Filesize { .. }
        | Value::Duration { .. }
        | Value::Date { .. }
        | Value::Range { .. }
        | Value::Closure { .. }
        | Value::Error { .. }
        | Value::Binary { .. }
        | Value::CellPath { .. }
        | Value::Custom { .. } => {
            return Err(LabeledError::new(format!(
                "Unsupported Nu type {:?}",
                nu_value.get_type()
            ))
            .with_label("not supported", nu_value.span()));
        }
    })
}

pub fn json_value_to_nu(json_value: &JsonValue) -> Result<Value, RenderError> {
    let span = Span::default();
    Ok(match json_value {
        JsonValue::Null => Value::nothing(span),
        JsonValue::Bool(val) => Value::bool(*val, span),
        JsonValue::Number(val) => {
            if let Some(val) = val.as_i64() {
                Value::int(val, span)
            } else if let Some(val) = val.as_f64() {
                Value::float(val, span)
            } else {
                return Err(RenderErrorReason::Other(format!(
                    "Unable to convert {val:?} to a Nu number"
                ))
                .into());
            }
        }
        JsonValue::String(val) => Value::string(val, span),
        JsonValue::Array(values) => Value::list(
            values
                .iter()
                .map(json_value_to_nu)
                .collect::<Result<_, _>>()?,
            span,
        ),
        JsonValue::Object(map) => Value::record(
            map.iter()
                .map(|(k, v)| Ok((k.to_owned(), json_value_to_nu(v)?)))
                .collect::<Result<_, RenderError>>()?,
            span,
        ),
    })
}
