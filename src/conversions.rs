use handlebars::{Context, RenderError, RenderErrorReason};
use nu_protocol::{LabeledError, PipelineData, ShellError, Span, Value};
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

pub fn nu_input_to_handlebars_context(
    input: PipelineData,
    call_span: Span,
) -> Result<Context, LabeledError> {
    match input {
        PipelineData::Empty => {
            Err(LabeledError::new("No data to render").with_label("needs input", call_span))
        }
        PipelineData::Value(value, _pipeline_metadata) => {
            Ok(Context::from(nu_value_to_json_value(&value)?))
        }
        PipelineData::ListStream(list_stream, _pipeline_metadata) => Ok(Context::from(
            nu_value_to_json_value(&list_stream.into_value()?)?,
        )),
        PipelineData::ByteStream(_byte_stream, _pipeline_metadata) => {
            Err(ShellError::PipelineMismatch {
                exp_input_type: "Structured Nu data".to_owned(),
                dst_span: call_span,
                src_span: _byte_stream.span(),
            }
            .into())
        }
    }
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
