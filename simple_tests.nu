use std/assert

assert equal "Hello world" ({name: "world"} | handlebars eval "Hello {{name}}")

let $tpl = handlebars new
| handlebars helper do-add {|a b| $a + $b}
| handlebars compile --text "{{a}} + {{b}} = {{do-add a b}}"

assert equal "1 + 2 = 3" ({a: 1, b: 2} | handlebars render $tpl)
