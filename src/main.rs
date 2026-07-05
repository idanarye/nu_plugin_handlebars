use nu_plugin::{MsgPackSerializer, serve_plugin};
use nu_plugin_handlebars::command::HandlebarsPlugin;

fn main() {
    serve_plugin(&HandlebarsPlugin, MsgPackSerializer);
}
