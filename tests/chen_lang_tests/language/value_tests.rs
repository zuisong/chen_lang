// Value system tests - testing the new unified value system
// including float operations, string operations, and type conversions
use crate::common::run_chen_lang_code;

#[test]
fn test_integer_arithmetic() {
    let code = r#"
local x = 5
local y = 3
print(x + y)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "8");
}

#[test]
fn test_float_arithmetic() {
    let code = r#"
local x = 3.14
local y = 2.0
print(x * y)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "6.28");
}

#[test]
fn test_string_concatenation() {
    let code = r#"
local hello = "Hello"
local world = " World"
print(hello + world)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "Hello World");
}

#[test]
fn test_mixed_type_arithmetic() {
    let code = r#"
local int_val = 5
local float_val = 2.5
local result = int_val + float_val
print(result)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "7.5");
}

#[test]
fn test_float_division() {
    let code = r#"
local x = 7.0
local y = 2.0
local result = x / y
print(result)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "3.5");
}

#[test]
fn test_negative_float() {
    let code = r#"
local x = -3.14
local y = 2.0
local result = x * y
print(result)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "-6.28");
}

#[test]
fn test_variable_assignment_with_float() {
    let code = r#"
local pi = 3.14159
local radius = 2.0
local area = pi * radius * radius
print(area)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "12.56636");
}

#[test]
fn test_string_with_numbers() {
    let code = r#"
local prefix = "Result: "
local number = 42
local message = prefix + "42"
print(message)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "Result: 42");
}

#[test]
fn test_complex_float_expression() {
    let code = r#"
local a = 1.5
local b = 2.0
local c = 3.0
local result = a + b * c - 0.5
print(result)
"#;
    let output = run_chen_lang_code(code).unwrap();
    // 1.5 + 2.0 * 3.0 - 0.5 = 1.5 + 6.0 - 0.5 = 7.0
    assert_eq!(output.trim(), "7");
}

#[test]
fn test_zero_float() {
    let code = r#"
local x = 0.0
local y = 5.0
local result = x + y
print(result)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "5");
}

#[test]
fn test_float_comparison() {
    let code = r#"
local a = 3.14
local b = 3.14
local equal = a == b
print(equal)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "true");
}

#[test]
fn test_mixed_comparison() {
    let code = r#"
local int_val = 5
local float_val = 5.0
local equal = int_val == float_val
print(equal)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "true");
}
