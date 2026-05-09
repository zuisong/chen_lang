use chen_lang::{RunOptions, run_captured_with_options};

use crate::common::run_chen_lang_code;

#[test]
fn struct_constructor_and_destructuring_match_work() {
    let code = r#"
struct Point { x: int, y: int }

let point = Point { x: 3, y: 4 }
let result = match point {
    Point { x, y } => x + y,
    _ => 0,
}
println(result)
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "7");
}

#[test]
fn enum_variant_and_payload_match_work() {
    let code = r#"
enum Option<T> { Some(T), None }

let present = Option.Some
present.value = 42
let missing = Option.None

let result = match present {
    Option.Some(v) => v,
    Option.None => 0,
    _ => -1,
}

let defaulted = match missing {
    Option.Some(v) => v,
    Option.None => 100,
    _ => -1,
}

println(result)
println(defaulted)
"#;

    let output = run_chen_lang_code(code).unwrap();
    let lines: Vec<&str> = output.trim().lines().collect();
    assert_eq!(lines, vec!["42", "100"]);
}

#[test]
fn impl_methods_are_available_through_method_syntax() {
    let code = r#"
struct Point { x: int, y: int }

impl Point {
    fn distance(&self, other: Point) -> float {
        let dx: float = self.x - other.x
        let dy: float = self.y - other.y
        return dx * dx + dy * dy
    }
}

let a = Point { x: 0, y: 0 }
let b = Point { x: 3, y: 4 }
println(a:distance(b))
"#;

    let output = run_chen_lang_code(code).unwrap();
    assert_eq!(output.trim(), "25");
}

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
def identity(x: int) -> int { return x }
let x: int = identity(1)
let io: object = import("stdlib/io")
io.println(x)
"#;

    let output = run_captured_with_options(code.to_string(), RunOptions { strict: true }).unwrap();
    assert_eq!(output.trim(), "1");
}
