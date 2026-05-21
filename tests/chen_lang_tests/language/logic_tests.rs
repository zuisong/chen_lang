use crate::common::run_chen_lang_code;

#[test]
fn test_boolean_operations() {
    let code = r#"
let a = true
let b = false
let result = a && b
console.log(result)
let result2 = a || b
console.log(result2)
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("false")); // a && b = false
    assert!(output.contains("true")); // a || b = true
}

#[test]
fn test_comparison_operations() {
    let code = r#"
let a = 5
let b = 3
let result = a > b
console.log(result)
let result2 = a == b
console.log(result2)
let result3 = a <= b
console.log(result3)
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert!(output.contains("true")); // a > b = true
    assert!(output.contains("false")); // a == b = false
    assert!(output.contains("false")); // a <= b = false
}
