use crate::common::run_chen_lang_code as run_captured;

#[test]
fn test_import_simple_debug() {
    let source = r#"
        console.log("Hello from test!")
        let mod = Chen.load("tests/fixtures/simple_test.chen.js")
        console.log("Module imported")
        let result = mod.test()
        console.log(result)
    "#;

    let output = run_captured(source).unwrap();
    assert!(output.contains("Hello from test!"));
    assert!(output.contains("Module imported"));
    assert!(output.contains("999"));
}

#[test]
fn test_access_imported_field() {
    let source = r#"
        let math = Chen.load("tests/fixtures/math_utils.chen.js")
        console.log("math object:")
        console.log(math)
        console.log("math.add:")
        console.log(math.add)
    "#;
    let output = run_captured(source).unwrap();
    println!("{}", output);
    assert!(output.contains("math object:"));
}
