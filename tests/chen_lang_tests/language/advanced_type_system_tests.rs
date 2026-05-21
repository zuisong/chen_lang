use chen_lang::{RunOptions, run_captured_with_options};

use crate::common::run_chen_lang_code;

#[test]
fn strict_mode_requires_explicit_annotations() {
    let code = r#"
let x = 1
"#;

    let err = run_captured_with_options(code.to_string(), RunOptions { strict: true }).unwrap_err();
    assert!(
        err.to_string()
            .contains("Strict mode requires explicit type annotation")
    );
}

#[test]
fn strict_mode_accepts_explicit_annotations() {
    let code = r#"
function identity(x: int) -> int { return x }
let x: int = identity(1)
console.log(x)
"#;

    let output = run_captured_with_options(code.to_string(), RunOptions { strict: true }).unwrap();
    assert_eq!(output.trim(), "1");
}

#[test]
fn type_annotations_in_assignment_work() {
    let code = r#"
let x: int = 10
x = 20
console.log(x)
"#;
    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "20");
}

#[test]
fn union_type_annotations_work() {
    let code = r#"
let x: int | string = 10
console.log(x)
x = "hello"
console.log(x)
"#;
    let output = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = output.trim().lines().collect();
    assert_eq!(lines[0], "10");
    assert_eq!(lines[1], "hello");
}
