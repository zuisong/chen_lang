// Value system tests - testing the new unified value system
// including float operations, string operations, and type conversions
mod common;
use common::run_chen_lang_code;

#[test]
fn test_integer_arithmetic() {
    let code = r#"
let x = 5
let y = 3
print(x + y)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "8");
}

#[test]
fn test_float_arithmetic() {
    let code = r#"
let x = 3.14
let y = 2.0
print(x * y)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "6.28");
}

#[test]
fn test_string_concatenation() {
    let code = r#"
let hello = "Hello"
let world = " World"
print(hello + world)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "Hello World");
}

#[test]
fn test_mixed_type_arithmetic() {
    let code = r#"
let int_val = 5
let float_val = 2.5
let result = int_val + float_val
print(result)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "7.5");
}

#[test]
fn test_float_division() {
    let code = r#"
let x = 7.0
let y = 2.0
let result = x / y
print(result)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "3.5");
}

#[test]
fn test_negative_float() {
    let code = r#"
let x = -3.14
let y = 2.0
let result = x * y
print(result)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "-6.28");
}

#[test]
fn test_variable_assignment_with_float() {
    let code = r#"
let pi = 3.14159
let radius = 2.0
let area = pi * radius * radius
print(area)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "12.56636");
}

#[test]
fn test_string_with_numbers() {
    let code = r#"
let prefix = "Result: "
let number = 42
let message = prefix + "42"
print(message)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "Result: 42");
}

#[test]
fn test_complex_float_expression() {
    let code = r#"
let a = 1.5
let b = 2.0
let c = 3.0
let result = a + b * c - 0.5
print(result)
"#;
    let output = run_chen_lang_code(code).unwrap();
    // 1.5 + 2.0 * 3.0 - 0.5 = 1.5 + 6.0 - 0.5 = 7.0
    assert_eq!(output.trim(), "7");
}

#[test]
fn test_zero_float() {
    let code = r#"
let x = 0.0
let y = 5.0
let result = x + y
print(result)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "5");
}

#[test]
fn test_float_comparison() {
    let code = r#"
let a = 3.14
let b = 3.14
let equal = a == b
print(equal)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "true");
}

#[test]
fn test_mixed_comparison() {
    let code = r#"
let int_val = 5
let float_val = 5.0
let equal = int_val == float_val
print(equal)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "true");
}
