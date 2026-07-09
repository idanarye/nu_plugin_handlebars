#!/usr/bin/env nu

let executable = cargo build -q --message-format json | lines | each {from json} | where reason == compiler-artifact | where executable != null | where target.name == nu_plugin_handlebars | last | get executable

nu --no-config-file --plugins $executable simple_tests.nu
