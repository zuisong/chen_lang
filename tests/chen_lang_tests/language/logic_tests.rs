use crate::common::run_chen_lang_code;

#[test]
fn test_boolean_operations() {
    let code = r#"
local a = 1
local b = 0
local result = a and b
print(result)
local result2 = a or b
print(result2)
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("false"));
    assert!(output.contains("true"));
}

#[test]
fn test_comparison_operations() {
    let code = r#"
local a = 5
local b = 3
local result = a > b
print(result)
local result2 = a == b
print(result2)
local result3 = a <= b
print(result3)
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("true"));
    assert!(output.contains("false"));
    assert!(output.contains("false"));
}
