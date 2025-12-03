mod common;
use common::run_chen_lang_code;

#[test]
fn test_simple_arithmetic() {
    let code = r#"
let i = 1
let j = 2
let k = i + j
print(k)
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "3");
}

#[test]
fn test_modulo_operation() {
    let code = r#"
let a = 10
let b = 3
let result = a % b
print(result)
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "1"); // 10 % 3 = 1
}

#[test]
fn test_complex_expression() {
    let code = r#"
let a = 2
let b = 3
let c = 4
let result = a + b * c
print(result)
let result2 = (a + b) * c
print(result2)
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("14")); // 2 + 3 * 4 = 14
    assert!(output.contains("20")); // (2 + 3) * 4 = 20
}
