use crate::declare_reference_type;
use handlebars::Template;

use super::RegistryReference;

#[derive(Debug)]
pub struct TemplateObject {
    pub registry: Option<RegistryReference>,
    pub template: Template,
}

declare_reference_type!(TemplateReference, TemplateObject, "HandlebarsTemplate");
