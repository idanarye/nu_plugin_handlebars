use handlebars::Handlebars;
use nu_protocol::{CustomValue, FromValue, ShellError, Span, Value};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::CustomReference;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HandlebarsRegistry(pub Uuid);

impl HandlebarsRegistry {
    pub(crate) fn into_value(self, span: Span) -> Value {
        Value::custom(Box::new(self), span)
    }
}

#[typetag::serde]
impl CustomValue for HandlebarsRegistry {
    fn clone_value(&self, span: Span) -> Value {
        self.clone().into_value(span)
    }

    fn type_name(&self) -> String {
        "HandlebarsRegistry".to_owned()
    }

    fn to_base_value(&self, span: Span) -> Result<Value, ShellError> {
        Ok(Value::string(
            format!("<HandlebarsRegistry:{}>", self.0),
            span,
        ))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn notify_plugin_on_drop(&self) -> bool {
        true
    }
}

impl CustomReference for HandlebarsRegistry {
    type Data = Handlebars<'static>;
    const NAME: &str = "HandlebarsRegistry";

    fn uuid(&self) -> &Uuid {
        &self.0
    }
}

impl FromValue for HandlebarsRegistry {
    fn from_value(v: Value) -> Result<Self, ShellError> {
        let custom = v.as_custom_value()?;
        custom
            .as_any()
            .downcast_ref()
            .ok_or_else(|| ShellError::TypeMismatch {
                err_message: format!("Expected HandlebarsRegistry, got {}", custom.type_name()),
                span: v.span(),
            })
            .cloned()
    }

    fn expected_type() -> nu_protocol::Type {
        nu_protocol::Type::custom("HandlebarsRegistry")
    }
}
