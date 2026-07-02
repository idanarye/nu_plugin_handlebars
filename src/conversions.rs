use nu_protocol::{LabeledError, Value};
use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue};

pub fn nu_value_to_json_value(nu_value: &Value) -> Result<JsonValue, LabeledError> {
    Ok(match nu_value {
        Value::Bool { val, .. } => JsonValue::Bool(*val),
        Value::Int { val, .. } => JsonValue::Number((*val).into()),
        Value::Float { val, .. } => JsonValue::Number(JsonNumber::from_f64(*val).unwrap()),
        Value::String { val, .. } => JsonValue::String(val.clone()),
        Value::Record { val, .. } => {
            let mut object = JsonMap::with_capacity(val.len());
            for (k, v) in val.iter() {
                object.insert(k.clone(), nu_value_to_json_value(v)?);
            }
            JsonValue::Object(object)
        }
        Value::List { vals, .. } => {
            let mut array = Vec::with_capacity(vals.len());
            for v in vals {
                array.push(nu_value_to_json_value(v)?);
            }
            JsonValue::Array(array)
        }
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
