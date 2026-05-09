use crate::common::run_chen_lang_code as run_captured;

#[test]
fn test_import_simple_debug() {
    let source = r#"
        let io = import("stdlib/io")  
        io.println("Hello from test!")
        let mod = import("tests/fixtures/simple_test.ch")
        io.println("Module imported")
        let result = mod.test()
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
        let io = import("stdlib/io")
        let math = import("tests/fixtures/math_utils.ch")
        io.println("math object:")
        io.println(math)
        io.println("math.add:")
        io.println(math.add)
    "#;
    let output = run_captured(source).unwrap();
    println!("{}", output);
    assert!(output.contains("math object:"));
}
