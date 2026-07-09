mod registry;
mod template;

use std::sync::RwLock;

use hashbrown::HashMap;
pub use registry::RegistryReference;
pub use template::{TemplateObject, TemplateReference};
use uuid::Uuid;

pub trait CustomReference: Clone {
    type Data;
    const NAME: &str;

    fn uuid(&self) -> &Uuid;
}

#[derive(Debug)]
pub struct CustomEntry<C: CustomReference> {
    pub reference: C,
    pub refcount: usize,
    pub data: C::Data,
}

#[derive(Default, Debug)]
pub struct CustomCollections {
    pub registries: RwLock<HashMap<Uuid, CustomEntry<RegistryReference>>>,
    pub templates: RwLock<HashMap<Uuid, CustomEntry<TemplateReference>>>,
}

impl CustomCollections {
    pub fn is_empty(&self) -> bool {
        self.registries.read().unwrap().is_empty() && self.templates.read().unwrap().is_empty()
    }
}

#[macro_export]
macro_rules! declare_reference_type {
    ($ref_type:ident, $obj_type:ty, $obj_name:literal) => {
        #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
        pub struct $ref_type(pub uuid::Uuid);

        impl $ref_type {
            pub(crate) fn into_value(self, span: nu_protocol::Span) -> nu_protocol::Value {
                nu_protocol::Value::custom(Box::new(self), span)
            }
        }

        #[typetag::serde]
        impl nu_protocol::CustomValue for $ref_type {
            fn clone_value(&self, span: nu_protocol::Span) -> nu_protocol::Value {
                self.clone().into_value(span)
            }

            fn type_name(&self) -> String {
                $obj_name.to_owned()
            }

            fn to_base_value(
                &self,
                span: nu_protocol::Span,
            ) -> Result<nu_protocol::Value, nu_protocol::ShellError> {
                Ok(nu_protocol::Value::string(
                    format!("<{}:{}>", $obj_name, self.0),
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

        impl $crate::custom_value::CustomReference for $ref_type {
            type Data = $obj_type;
            const NAME: &str = $obj_name;

            fn uuid(&self) -> &uuid::Uuid {
                &self.0
            }
        }

        impl nu_protocol::FromValue for $ref_type {
            fn from_value(v: nu_protocol::Value) -> Result<Self, nu_protocol::ShellError> {
                let custom = v.as_custom_value()?;
                custom
                    .as_any()
                    .downcast_ref()
                    .ok_or_else(|| nu_protocol::ShellError::TypeMismatch {
                        err_message: format!("Expected {}, got {}", $obj_name, custom.type_name()),
                        span: v.span(),
                    })
                    .cloned()
            }

            fn expected_type() -> nu_protocol::Type {
                nu_protocol::Type::custom($obj_name)
            }
        }
    };
}
