use crate::common::run_chen_lang_code;

#[test]
fn test_string_len() {
    let code = r#"
    let s = "hello"
    println(s.len())
    println("abc".len())
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("5"));
    assert!(output.contains("3"));
}

#[test]
fn test_string_upper_lower() {
    let code = r#"
    let s = "Hello"
    println(s.upper())
    println(s.lower())
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("HELLO"));
    assert!(output.contains("hello"));
}

#[test]
fn test_string_trim() {
    let code = r#"
    let s = "  hello world  "
    println("'" + s.trim() + "'")
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("'hello world'"));
}

#[test]
fn test_string_metadata() {
    let code = r#"
    let s = "test"
    println(s.__type)
    "#;
    let output = run_chen_lang_code(code).expect("Execution failed");
    assert!(output.contains("String"));
}
