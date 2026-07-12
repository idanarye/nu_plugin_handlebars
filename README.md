[![Build Status](https://github.com/idanarye/nu_plugin_handlebars/workflows/CI/badge.svg)](https://github.com/idanarye/nu_plugin_handlebars/actions)
[![Latest Version](https://img.shields.io/crates/v/nu_plugin_handlebars.svg)](https://crates.io/crates/nu_plugin_handlebars)

# nu_plugin_handlebars

This is a [Nushell](https://nushell.sh/) plugin that adds integrates [Handlebars](https://handlebarsjs.com/) template engine, powered by the [`handlebars`](https://github.com/sunng87/handlebars-rust) crate.

# Features

* Redner Handlebars templates from Nushell.
* The data for Handlebars is Nu values - no need to serialize them.
* Supports [partials](https://handlebarsjs.com/guide/partials.html).
* Supports [helpers](https://handlebarsjs.com/guide/expressions.html#helpers).

## Planned features

* [Block helpers](https://handlebarsjs.com/guide/block-helpers.html).

## Installing

Install the crate using:

```nu
cargo install nu_plugin_handlebars
```

Then register the plugin using (this must be done inside Nushell):

```nu
plugin add ~/.cargo/bin/nu_plugin_handlebars
```

## Usage

```nu
let hb = handlebars new
$hb | handlebars helper add {|a b| $a + $b}
$hb | handlebars partial addition --text "{{x}} + {{y}}"
let tpl = $hb | handlebars compile --text "{{>addition}} = {{add x y}}"
{x: 1, y: 2} | handlebars render $tpl
# This prints `1 + 2 = 3`
```

`handlebars helper` and `handlebars partial` also return the registry, so the entire registry creation and template compilation can be pipelined:

```nu
let tpl = handlebars new
| handlebars helper add {|a b| $a + $b}
| handlebars partial addition --text "{{x}} + {{y}}"
| handlebars compile --text "{{>addition}} = {{add x y}}"
```

* Commands that accept a string via a `--text` flag can also read from a file using a `--file` arg.
* There is an `handlebars eval` command that can compile and render a template in a single command.
