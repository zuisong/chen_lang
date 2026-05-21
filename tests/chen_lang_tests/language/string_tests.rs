use crate::common::run_chen_lang_code;

#[test]
fn test_string_len() {
    let code = r#"
    let s = "hello"
    console.log(s.length)
    console.log("abc".length)
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("5"));
    assert!(output.contains("3"));
}

#[test]
fn test_string_upper_lower() {
    let code = r#"
    let s = "Hello"
    console.log(s.toUpperCase())
    console.log(s.toLowerCase())
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("HELLO"));
    assert!(output.contains("hello"));
}

#[test]
fn test_string_trim() {
    let code = r#"
    let s = "  hello world  "
    console.log("'" + s.trim() + "'")
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("'hello world'"));
}

#[test]
fn test_string_metadata() {
    let code = r#"
    let s = "test"
    console.log(s.__type)
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("String"));
}
