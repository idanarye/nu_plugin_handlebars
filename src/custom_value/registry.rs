use crate::declare_reference_type;
use handlebars::Handlebars;

declare_reference_type!(RegistryReference, Handlebars<'static>, "HandlebarsRegistry");
