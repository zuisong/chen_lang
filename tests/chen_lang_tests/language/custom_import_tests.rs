use crate::common::run_chen_lang_code as run_captured;

#[test]
fn test_import_custom_module_simple() {
    let source = r#"
        local io = require("stdlib/io")
        local mod = require("tests/fixtures/temp_module.chen.luau")
        io.write(mod.name)
        io.write(mod.greet("World"))
    "#;

    let output = run_captured(&source.to_string()).unwrap();

    assert!(output.contains("Module"));
    assert!(output.contains("Hello, World from Module"));
}

#[test]
fn test_import_custom_module_relative_path() {
    // Note: Paths are currently relative to CWD (project root during cargo test)
    let source = r#"
        local io = require("stdlib/io")
        local math = require("tests/fixtures/math_utils.chen.luau")
        io.write(math.add(10, 20))
    "#;

    let output = run_captured(&source.to_string()).unwrap();

    assert!(output.contains("30"));
}

#[test]
fn test_import_custom_module_caching() {
    let source = r#"
        local m1 = require("tests/fixtures/cached_module.chen.luau")
        local m2 = require("tests/fixtures/cached_module.chen.luau")
    "#;

    let output = run_captured(&source.to_string()).unwrap();

    // "Module Loaded" should appear only once if caching works
    let matches: Vec<_> = output.matches("Module Loaded").collect();
    assert_eq!(matches.len(), 1, "Module should be loaded exactly once due to caching");
}

#[test]
fn test_call_imported_function() {
    let source = r#"
        local io = require("stdlib/io")
        local math = require("tests/fixtures/math_utils.chen.luau")
        io.write("Before call")
        local result = math.add(10, 20)
        io.write("After call")
        io.write(result)
    "#;
    let output = run_captured(&source.to_string()).unwrap();
    assert!(output.contains("Before call"));
    assert!(output.contains("After call"));
    assert!(output.contains("30"));
}
