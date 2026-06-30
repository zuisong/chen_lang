use crate::common::run_chen_lang_code as run_captured;

#[test]
fn test_import_simple_debug() {
    let source = r#"
        local io = require("stdlib/io")  
        io.write("Hello from test!")
        local mod = require("tests/fixtures/simple_test.chen.luau")
        io.write("Module imported")
        local result = mod.test()
        io.write(result)
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
        io.write("math object:")
        io.write(math)
        io.write("math.add:")
        io.write(math.add)
    "#;
    let output = run_captured(source).unwrap();
    println!("{}", output);
    assert!(output.contains("math object:"));
}
