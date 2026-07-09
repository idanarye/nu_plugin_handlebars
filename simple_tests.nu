use std/assert

def test [caption: string, body: closure] {
    print $"($caption):"
    timeit --output $body | print
}

test "Simple eval" {
    assert equal "Hello world" ({name: "world"} | handlebars eval "Hello {{name}}")
}

test "Helper" {
    let $tpl = handlebars new
    | handlebars helper do-add {|a b| $a + $b}
    | handlebars compile --text "{{a}} + {{b}} = {{do-add a b}}"

    assert equal "1 + 2 = 3" ({a: 1, b: 2} | handlebars render $tpl)
}

test "Partial" {
    let $hb = handlebars new
    | handlebars partial greet --text "Hello {{name}}"

    assert equal "Hello world, I am me" ({name: "world", my_name: "me"} | handlebars eval "{{>greet}}, I am {{my_name}}" --with $hb)
}

test "From files" {
    let partial_file = mktemp --suffix ".hbs"
    "mr. {{name}}" o> $partial_file
    let template_file = mktemp --suffix ".hbs"
    "Hello {{>greet-target}}" o> $template_file
    let $tpl = handlebars new
    | handlebars partial greet-target --file $partial_file
    | handlebars compile --file $template_file
    assert equal "Hello mr. World" ({name: "World"} | handlebars render $tpl)
}
