use crate::common::run_chen_lang_code as run_captured;

#[test]
fn test_import_simple_debug() {
    let source = r#"
        local io = require("stdlib/io")  
        io.println("Hello from test!")
        local mod = require("tests/fixtures/simple_test.chen.luau")
        io.println("Module imported")
        local result = mod.test()
        io.println(result)
    "#;

    let output = run_captured(source).unwrap();
    assert!(output.contains("Hello from test!"));
    assert!(output.contains("Module imported"));
    assert!(output.contains("999"));
}

#[test]
fn test_access_imported_field() {
    let source = r#"
        local io = require("stdlib/io")
        local math = require("tests/fixtures/math_utils.chen.luau")
        io.println("math object:")
        io.println(math)
        io.println("math.add:")
        io.println(math.add)
    "#;
    let output = run_captured(source).unwrap();
    println!("{}", output);
    assert!(output.contains("math object:"));
}
