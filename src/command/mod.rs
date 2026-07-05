use nu_plugin::Plugin;

pub mod eval;

pub struct HandlebarsPlugin;

impl Plugin for HandlebarsPlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_owned()
    }

    fn commands(&self) -> Vec<Box<dyn nu_plugin::PluginCommand<Plugin = Self>>> {
        vec![Box::new(eval::HandlebarsEvalCommand)]
    }
}
